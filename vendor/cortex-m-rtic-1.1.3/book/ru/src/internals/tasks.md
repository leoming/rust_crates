# Программные задачи

RTIC поддерживает программные и аппаратные задачи. Каждая аппаратная задача
назначается на отдельный обработчик прерывания. С другой стороны, несколько
программных задач могут управляться одним обработчиком прерывания --
это сделано, чтобы минимизировать количество обработчиков прерывания,
используемых фреймворком.

Фреймворк группирует задачи, для которых вызывается `spawn` по уровню приоритета,
и генерирует один *диспетчер задачи* для каждого уровня приоритета.
Каждый диспетчер запускается на отдельном обработчике прерывания,
а приоритет этого обработчика прерывания устанавливается так, чтобы соответствовать
уровню приоритета задач, управляемых диспетчером.

Каждый диспетчер задач хранит *очередь* задач, *готовых* к выполнению;
эта очередь называется *очередью готовности*. Вызов программной задачи состоит
из добавления записи в очередь и вызова прерывания, который запускает соответствующий
диспетчер задач. Каждая запись в эту очередь содержит метку (`enum`),
которая идентифицирует задачу, которую необходимо выполнить и *указатель*
на сообщение, передаваемое задаче.

Очередь готовности - неблокируемая очередь типа SPSC (один производитель - один потребитель).
Диспетчер задач владеет конечным потребителем в очереди; конечным производителем
считается ресурс, за который соперничают задачи, которые могут вызывать (`spawn`) другие задачи.

## Дисметчер задач

Давайте сначала глянем на код, генерируемый фреймворком для диспетчеризации задач.
Рассмотрим пример:

``` rust
#[rtic::app(device = ..)]
mod app {
    // ..

    #[interrupt(binds = UART0, priority = 2, spawn = [bar, baz])]
    fn foo(c: foo::Context) {
        foo.spawn.bar().ok();

        foo.spawn.baz(42).ok();
    }

    #[task(capacity = 2, priority = 1)]
    fn bar(c: bar::Context) {
        // ..
    }

    #[task(capacity = 2, priority = 1, resources = [X])]
    fn baz(c: baz::Context, input: i32) {
        // ..
    }

    extern "C" {
        fn UART1();
    }
}
```

Фреймворк создает следующий диспетчер задач, состоящий из обработчика прерывания и очереди готовности:

``` rust
fn bar(c: bar::Context) {
    // .. пользовательский код ..
}

mod app {
    use heapless::spsc::Queue;
    use cortex_m::register::basepri;

    struct Ready<T> {
        task: T,
        // ..
    }

    /// вызываемые (`spawn`) задачи, выполняющиеся с уровнем приоритета `1`
    enum T1 {
        bar,
        baz,
    }

    // очередь готовности диспетчера задач
    // `5-1=4` - представляет собой емкость этой очереди
    static mut RQ1: Queue<Ready<T1>, 5> = Queue::new();

    // обработчик прерывания, выбранный для диспетчеризации задач с приоритетом `1`
    #[no_mangle]
    unsafe UART1() {
        // приоритет данного обработчика прерывания
        const PRIORITY: u8 = 1;

        let snapshot = basepri::read();

        while let Some(ready) = RQ1.split().1.dequeue() {
            match ready.task {
                T1::bar => {
                    // **ПРИМЕЧАНИЕ** упрощенная реализация

                    // используется для отслеживания динамического приоритета
                    let priority = Cell::new(PRIORITY);

                    // вызов пользовательского кода
                    bar(bar::Context::new(&priority));
                }

                T1::baz => {
                    // рассмотрим `baz` позднее
                }
            }
        }

        // инвариант BASEPRI
        basepri::write(snapshot);
    }
}
```

## Вызов задачи

Интерфейс `spawn` предоставлен пользователю как методы структурв `Spawn`.
Для каждой задачи существует своя структура `Spawn`.

Код `Spawn`, генерируемый фреймворком для предыдущего примера выглядит так:

``` rust
mod foo {
    // ..

    pub struct Context<'a> {
        pub spawn: Spawn<'a>,
        // ..
    }

    pub struct Spawn<'a> {
        // отслеживает динамический приоритет задачи
        priority: &'a Cell<u8>,
    }

    impl<'a> Spawn<'a> {
        // `unsafe` и спрятано, поскольку сы не хотит, чтобы пользователь вмешивался сюда
        #[doc(hidden)]
        pub unsafe fn priority(&self) -> &Cell<u8> {
            self.priority
        }
    }
}

mod app {
    // ..

    // Поиск максимального приоритета для конечного производителя `RQ1`
    const RQ1_CEILING: u8 = 2;

    // используется, чтобы отследить сколько еще сообщений для `bar` можно поставить в очередь
    // `3-1=2` - емкость задачи `bar`; максимум 2 экземпляра можно добавить в очередь
    // эта очередь заполняется фреймворком до того, как запустится `init`
    static mut bar_FQ: Queue<(), 3> = Queue::new();

    // Поиск максимального приоритета для конечного потребителя `bar_FQ`
    const bar_FQ_CEILING: u8 = 2;

    // приоритет-ориентированная критическая секция
    //
    // это запускае переданное замыкание `f` с динамическим приоритетом не ниже
    // `ceiling`
    fn lock(priority: &Cell<u8>, ceiling: u8, f: impl FnOnce()) {
        // ..
    }

    impl<'a> foo::Spawn<'a> {
        /// Вызывает задачу `bar`
        pub fn bar(&self) -> Result<(), ()> {
            unsafe {
                match lock(self.priority(), bar_FQ_CEILING, || {
                    bar_FQ.split().1.dequeue()
                }) {
                    Some(()) => {
                        lock(self.priority(), RQ1_CEILING, || {
                            // помещаем задачу в очередь готовности
                            RQ1.split().1.enqueue_unchecked(Ready {
                                task: T1::bar,
                                // ..
                            })
                        });

                        // вызываем прерывание, которое запускает диспетчер задач
                        rtic::pend(Interrupt::UART0);
                    }

                    None => {
                        // достигнута максимальная вместительность; неудачный вызов
                        Err(())
                    }
                }
            }
        }
    }
}
```

Использование `bar_FQ` для ограничения числа задач `bar`, которые могут бы вызваны,
может показаться искусственным, но это будет иметь больше смысла, когда мы поговорим
о вместительности задач.

## Сообщения

Мы пропустили, как на самом деле работает передача сообщений, поэтому давайте вернемся
к реализации `spawn`, но в этот раз для задачи `baz`, которая принимает сообщение типа `u64`.

``` rust
fn baz(c: baz::Context, input: u64) {
    // .. пользовательский код ..
}

mod app {
    // ..

    // Теперь мы покажем все содержимое структуры `Ready`
    struct Ready {
        task: Task,
        // индекс сообщения; используется с буфером `INPUTS`
        index: u8,
    }

    // память, зарезервированная для хранения сообщений, переданных `baz`
    static mut baz_INPUTS: [MaybeUninit<u64>; 2] =
        [MaybeUninit::uninit(), MaybeUninit::uninit()];

    // список свободной памяти: используется для отслеживания свободных ячеек в массиве `baz_INPUTS`
    // эта очередь инициализируется значениями `0` и `1` перед запуском `init`
    static mut baz_FQ: Queue<u8, 3> = Queue::new();

    // Поиск максимального приоритета для конечного потребителя `baz_FQ`
    const baz_FQ_CEILING: u8 = 2;

    impl<'a> foo::Spawn<'a> {
        /// Spawns the `baz` task
        pub fn baz(&self, message: u64) -> Result<(), u64> {
            unsafe {
                match lock(self.priority(), baz_FQ_CEILING, || {
                    baz_FQ.split().1.dequeue()
                }) {
                    Some(index) => {
                        // ПРИМЕЧАНИЕ: `index` - владеющий указатель на ячейку буфера
                        baz_INPUTS[index as usize].write(message);

                        lock(self.priority(), RQ1_CEILING, || {
                            // помещаем задачу в очередь готовности
                            RQ1.split().1.enqueue_unchecked(Ready {
                                task: T1::baz,
                                index,
                            });
                        });

                        // вызываем прерывание, которое запускает диспетчер задач
                        rtic::pend(Interrupt::UART0);
                    }

                    None => {
                        // достигнута максимальная вместительность; неудачный вызов
                        Err(message)
                    }
                }
            }
        }
    }
}
```

А теперь давайте взглянем на настоящую реализацию диспетчера задач:

``` rust
mod app {
    // ..

    #[no_mangle]
    unsafe UART1() {
        const PRIORITY: u8 = 1;

        let snapshot = basepri::read();

        while let Some(ready) = RQ1.split().1.dequeue() {
            match ready.task {
                Task::baz => {
                    // ПРИМЕЧАНИЕ: `index` - владеющий указатель на ячейку буфера
                    let input = baz_INPUTS[ready.index as usize].read();

                    // сообщение было прочитано, поэтому можно вернуть ячейку обратно
                    // чтобы освободить очередь
                    // (диспетчер задач имеет эксклюзивный доступ к
                    // последнему элементу очереди)
                    baz_FQ.split().0.enqueue_unchecked(ready.index);

                    let priority = Cell::new(PRIORITY);
                    baz(baz::Context::new(&priority), input)
                }

                Task::bar => {
                    // выглядит также как ветка для `baz`
                }

            }
        }

        // инвариант BASEPRI
        basepri::write(snapshot);
    }
}
```

`INPUTS` плюс `FQ`, список свободной памяти равняется эффективному пулу памяти.
Однако, вместо того *список свободной памяти* (связный список), чтобы отслеживать
пустые ячейки в буфере `INPUTS`, мы используем SPSC очередь; это позволяет нам
уменьшить количество критических секций.
На самом деле благодаря этому выбору код диспетчера задач неблокируемый.

## Вместительность очереди

Фреймворк RTIC использует несколько очередей, такие как очереди готовности и
списки свободной памяти. Когда список свободной памяти пуст, попытка выызова
(`spawn`) задачи приводит к ошибке; это условие проверяется во время выполнения.
Не все операции, произвожимые фреймворком с этими очередями проверяют их
пустоту / наличие места. Например, возвращение ячейки списка свободной памяти
(см. диспетчер задач) не проверяется, поскольку есть фиксированное количество
таких ячеек циркулирующих в системе, равное вместительности списка свободной памяти.
Аналогично, добавление записи в очередь готовности (см. `Spawn`) не проверяется,
потому что вместительность очереди выбрана фреймворком.

Пользователи могут задавать вместительность программных задач;
эта вместительность - максимальное количество сообщений, которые можно
послать указанной задаче от задачи более высоким приоритетом до того,
как `spawn` вернет ошибку. Эта определяемая пользователем иместительность -
размер списка свободной памяти задачи (например `foo_FQ`), а также размер массива,
содержащего входные данные для задачи (например `foo_INPUTS`).

Вместительность очереди готовности (например `RQ1`) вычисляется как *сумма*
вместительностей всех задач, управляемх диспетчером; эта сумма является также
количеством сообщений, которые очередь может хранить в худшем сценарии, когда
все возможные сообщения были посланы до того, как диспетчер задач получает шанс
на запуск. По этой причине получение ячейки списка свободной памяти при любой
операции `spawn` приводит к тому, что очередь готовности еще не заполнена,
поэтому вставка записи в список готовности может пропустить проверку "полна ли очередь?".

В нашем запущенном примере задача `bar` не принимает входных данных, поэтому
мы можем пропустить проверку как `bar_INPUTS`, так и `bar_FQ` и позволить
пользователю посылать неограниченное число сообщений задаче, но если бы мы сделали это,
было бы невозможно превысить вместительность для `RQ1`, что позволяет нам
пропустить проверку "полна ли очередь?" при вызове задачи `baz`.
В разделе о [очереди таймера](timer-queue.html) мы увидим как
список свободной памяти используется для задач без входных данных.

## Анализ приоритетов

Очереди, использемые внутри интерфейса `spawn`, рассматриваются как обычные ресурсы
и для них тоже работает анализ приоритетов. Важно заметить, что это SPSC очереди,
и только один из конечных элементов становится ресурсом; другим конечным элементом
владеет диспетчер задач.

Рассмотрим следующий пример:

``` rust
#[rtic::app(device = ..)]
mod app {
    #[idle(spawn = [foo, bar])]
    fn idle(c: idle::Context) -> ! {
        // ..
    }

    #[task]
    fn foo(c: foo::Context) {
        // ..
    }

    #[task]
    fn bar(c: bar::Context) {
        // ..
    }

    #[task(priority = 2, spawn = [foo])]
    fn baz(c: baz::Context) {
        // ..
    }

    #[task(priority = 3, spawn = [bar])]
    fn quux(c: quux::Context) {
        // ..
    }
}
```

Вот как будет проходить анализ приоритетов:

- `idle` (prio = 0) и `baz` (prio = 2) соревнуются за конечный потребитель
  `foo_FQ`; это приводит к максимальному приоритету `2`.

- `idle` (prio = 0) и `quux` (prio = 3) соревнуются за конечный потребитель
  `bar_FQ`; это приводит к максимальному приоритету `3`.

- `idle` (prio = 0), `baz` (prio = 2) и `quux` (prio = 3) соревнуются за
  конечный производитель `RQ1`; это приводит к максимальному приоритету `3`
