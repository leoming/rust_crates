use byteorder::ByteOrder;

/// Timestamp resolution of the pcap
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TsResolution {
    MicroSecond,
    NanoSecond
}

/// Endianness of the pcap
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Endianness {
    Big,
    Little
}

impl Endianness {

    pub fn is_little(self) -> bool {
        match self {
            Endianness::Big => false,
            Endianness::Little => true
        }
    }

    pub fn is_big(self) -> bool {
        match self {
            Endianness::Big => true,
            Endianness::Little => false
        }
    }

    pub fn new<B: ByteOrder>() -> Self {

        if B::read_u32(&[0,0,0,1]) == 1 {
            Endianness::Big
        }
        else {
            Endianness::Little
        }
    }
}

/// Data link type
///
/// The link-layer header type specifies the first protocol of the packet.
///
/// See [http://www.tcpdump.org/linktypes.html](http://www.tcpdump.org/linktypes.html)
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DataLink {

    NULL,
    ETHERNET,
    AX25,
    IEEE802_5,
    ARCNET_BSD,
    SLIP,
    PPP,
    FDDI,
    PPP_HDLC,
    PPP_ETHER,
    ATM_RFC1483,
    RAW,
    C_HDLC,
    IEEE802_11,
    FRELAY,
    LOOP,
    LINUX_SLL,
    LTALK,
    PFLOG,
    IEEE802_11_PRISM,
    IP_OVER_FC,
    SUNATM,
    IEEE802_11_RADIOTAP,
    ARCNET_LINUX,
    APPLE_IP_OVER_IEEE1394,
    MTP2_WITH_PHDR,
    MTP2,
    MTP3,
    SCCP,
    DOCSIS,
    LINUX_IRDA,
    USER0,
    USER1,
    USER2,
    USER3,
    USER4,
    USER5,
    USER6,
    USER7,
    USER8,
    USER9,
    USER10,
    USER11,
    USER12,
    USER13,
    USER14,
    USER15,
    IEEE802_11_AVS,
    BACNET_MS_TP,
    PPP_PPPD,
    GPRS_LLC,
    GPF_T,
    GPF_F,
    LINUX_LAPD,
    BLUETOOTH_HCI_H4,
    USB_LINUX,
    PPI,
    IEEE802_15_4,
    SITA,
    ERF,
    BLUETOOTH_HCI_H4_WITH_PHDR,
    AX25_KISS,
    LAPD,
    PPP_WITH_DIR,
    C_HDLC_WITH_DIR,
    FRELAY_WITH_DIR,
    IPMB_LINUX,
    IEEE802_15_4_NONASK_PHY,
    USB_LINUX_MMAPPED,
    FC_2,
    FC_2_WITH_FRAME_DELIMS,
    IPNET,
    CAN_SOCKETCAN,
    IPV4,
    IPV6,
    IEEE802_15_4_NOFCS,
    DBUS,
    DVB_CI,
    MUX27010,
    STANAG_5066_D_PDU,
    NFLOG,
    NETANALYZER,
    NETANALYZER_TRANSPARENT,
    IPOIB,
    MPEG_2_TS,
    NG40,
    NFC_LLCP,
    INFINIBAND,
    SCTP,
    USBPCAP,
    RTAC_SERIAL,
    BLUETOOTH_LE_LL,
    NETLINK,
    BLUETOOTH_LINUX_MONITOR,
    BLUETOOTH_BREDR_BB,
    BLUETOOTH_LE_LL_WITH_PHDR,
    PROFIBUS_DL,
    PKTAP,
    EPON,
    IPMI_HPM_2,
    ZWAVE_R1_R2,
    ZWAVE_R3,
    WATTSTOPPER_DLM,
    ISO_14443,
    RDS,
    USB_DARWIN,
    SDLC,

    Unknown(u32)
}

impl From<u32> for DataLink {

    fn from(n: u32) -> DataLink {

        match n {
            0 => DataLink::NULL,
            1 => DataLink::ETHERNET,
            3 => DataLink::AX25,
            6 => DataLink::IEEE802_5,
            7 => DataLink::ARCNET_BSD,
            8 => DataLink::SLIP,
            9 => DataLink::PPP,
            10 => DataLink::FDDI,
            50 => DataLink::PPP_HDLC,
            51 => DataLink::PPP_ETHER,
            100 => DataLink::ATM_RFC1483,
            101 => DataLink::RAW,
            104 => DataLink::C_HDLC,
            105 => DataLink::IEEE802_11,
            107 => DataLink::FRELAY,
            108 => DataLink::LOOP,
            113 => DataLink::LINUX_SLL,
            114 => DataLink::LTALK,
            117 => DataLink::PFLOG,
            119 => DataLink::IEEE802_11_PRISM,
            122 => DataLink::IP_OVER_FC,
            123 => DataLink::SUNATM,
            127 => DataLink::IEEE802_11_RADIOTAP,
            129 => DataLink::ARCNET_LINUX,
            138 => DataLink::APPLE_IP_OVER_IEEE1394,
            139 => DataLink::MTP2_WITH_PHDR,
            140 => DataLink::MTP2,
            141 => DataLink::MTP3,
            142 => DataLink::SCCP,
            143 => DataLink::DOCSIS,
            144 => DataLink::LINUX_IRDA,
            147 => DataLink::USER0,
            148 => DataLink::USER1,
            149 => DataLink::USER2,
            150 => DataLink::USER3,
            151 => DataLink::USER4,
            152 => DataLink::USER5,
            153 => DataLink::USER6,
            154 => DataLink::USER7,
            155 => DataLink::USER8,
            156 => DataLink::USER9,
            157 => DataLink::USER10,
            158 => DataLink::USER11,
            159 => DataLink::USER12,
            160 => DataLink::USER13,
            161 => DataLink::USER14,
            162 => DataLink::USER15,
            163 => DataLink::IEEE802_11_AVS,
            165 => DataLink::BACNET_MS_TP,
            166 => DataLink::PPP_PPPD,
            169 => DataLink::GPRS_LLC,
            170 => DataLink::GPF_T,
            171 => DataLink::GPF_F,
            177 => DataLink::LINUX_LAPD,
            187 => DataLink::BLUETOOTH_HCI_H4,
            189 => DataLink::USB_LINUX,
            192 => DataLink::PPI,
            195 => DataLink::IEEE802_15_4,
            196 => DataLink::SITA,
            197 => DataLink::ERF,
            201 => DataLink::BLUETOOTH_HCI_H4_WITH_PHDR,
            202 => DataLink::AX25_KISS,
            203 => DataLink::LAPD,
            204 => DataLink::PPP_WITH_DIR,
            205 => DataLink::C_HDLC_WITH_DIR,
            206 => DataLink::FRELAY_WITH_DIR,
            209 => DataLink::IPMB_LINUX,
            215 => DataLink::IEEE802_15_4_NONASK_PHY,
            220 => DataLink::USB_LINUX_MMAPPED,
            224 => DataLink::FC_2,
            225 => DataLink::FC_2_WITH_FRAME_DELIMS,
            226 => DataLink::IPNET,
            227 => DataLink::CAN_SOCKETCAN,
            228 => DataLink::IPV4,
            229 => DataLink::IPV6,
            230 => DataLink::IEEE802_15_4_NOFCS,
            231 => DataLink::DBUS,
            235 => DataLink::DVB_CI,
            236 => DataLink::MUX27010,
            237 => DataLink::STANAG_5066_D_PDU,
            239 => DataLink::NFLOG,
            240 => DataLink::NETANALYZER,
            241 => DataLink::NETANALYZER_TRANSPARENT,
            242 => DataLink::IPOIB,
            243 => DataLink::MPEG_2_TS,
            244 => DataLink::NG40,
            245 => DataLink::NFC_LLCP,
            247 => DataLink::INFINIBAND,
            248 => DataLink::SCTP,
            249 => DataLink::USBPCAP,
            250 => DataLink::RTAC_SERIAL,
            251 => DataLink::BLUETOOTH_LE_LL,
            253 => DataLink::NETLINK,
            254 => DataLink::BLUETOOTH_LINUX_MONITOR,
            255 => DataLink::BLUETOOTH_BREDR_BB,
            256 => DataLink::BLUETOOTH_LE_LL_WITH_PHDR,
            257 => DataLink::PROFIBUS_DL,
            258 => DataLink::PKTAP,
            259 => DataLink::EPON,
            260 => DataLink::IPMI_HPM_2,
            261 => DataLink::ZWAVE_R1_R2,
            262 => DataLink::ZWAVE_R3,
            263 => DataLink::WATTSTOPPER_DLM,
            264 => DataLink::ISO_14443,
            265 => DataLink::RDS,
            266 => DataLink::USB_DARWIN,
            268 => DataLink::SDLC,

            _ => DataLink::Unknown(n)
        }
    }
}

impl From<DataLink> for u32 {

    fn from(link: DataLink) -> u32 {

        match link {

            DataLink::NULL => 0,
            DataLink::ETHERNET => 1,
            DataLink::AX25 => 3,
            DataLink::IEEE802_5 => 6,
            DataLink::ARCNET_BSD => 7,
            DataLink::SLIP => 8,
            DataLink::PPP => 9,
            DataLink::FDDI => 10,
            DataLink::PPP_HDLC => 50,
            DataLink::PPP_ETHER => 51,
            DataLink::ATM_RFC1483 => 100,
            DataLink::RAW => 101,
            DataLink::C_HDLC => 104,
            DataLink::IEEE802_11 => 105,
            DataLink::FRELAY => 107,
            DataLink::LOOP => 108,
            DataLink::LINUX_SLL => 113,
            DataLink::LTALK => 114,
            DataLink::PFLOG => 117,
            DataLink::IEEE802_11_PRISM => 119,
            DataLink::IP_OVER_FC => 122,
            DataLink::SUNATM => 123,
            DataLink::IEEE802_11_RADIOTAP => 127,
            DataLink::ARCNET_LINUX => 129,
            DataLink::APPLE_IP_OVER_IEEE1394 => 138,
            DataLink::MTP2_WITH_PHDR => 139,
            DataLink::MTP2 => 140,
            DataLink::MTP3 => 141,
            DataLink::SCCP => 142,
            DataLink::DOCSIS => 143,
            DataLink::LINUX_IRDA => 144,
            DataLink::USER0 => 147,
            DataLink::USER1 => 148,
            DataLink::USER2 => 149,
            DataLink::USER3 => 150,
            DataLink::USER4 => 151,
            DataLink::USER5 => 152,
            DataLink::USER6 => 153,
            DataLink::USER7 => 154,
            DataLink::USER8 => 155,
            DataLink::USER9 => 156,
            DataLink::USER10 => 157,
            DataLink::USER11 => 158,
            DataLink::USER12 => 159,
            DataLink::USER13 => 160,
            DataLink::USER14 => 161,
            DataLink::USER15 => 162,
            DataLink::IEEE802_11_AVS => 163,
            DataLink::BACNET_MS_TP => 165,
            DataLink::PPP_PPPD => 166,
            DataLink::GPRS_LLC => 169,
            DataLink::GPF_T => 170,
            DataLink::GPF_F => 171,
            DataLink::LINUX_LAPD => 177,
            DataLink::BLUETOOTH_HCI_H4 => 187,
            DataLink::USB_LINUX => 189,
            DataLink::PPI => 192,
            DataLink::IEEE802_15_4 => 195,
            DataLink::SITA => 196,
            DataLink::ERF => 197,
            DataLink::BLUETOOTH_HCI_H4_WITH_PHDR => 201,
            DataLink::AX25_KISS => 202,
            DataLink::LAPD => 203,
            DataLink::PPP_WITH_DIR => 204,
            DataLink::C_HDLC_WITH_DIR => 205,
            DataLink::FRELAY_WITH_DIR => 206,
            DataLink::IPMB_LINUX => 209,
            DataLink::IEEE802_15_4_NONASK_PHY => 215,
            DataLink::USB_LINUX_MMAPPED => 220,
            DataLink::FC_2 => 224,
            DataLink::FC_2_WITH_FRAME_DELIMS => 225,
            DataLink::IPNET => 226,
            DataLink::CAN_SOCKETCAN => 227,
            DataLink::IPV4 => 228,
            DataLink::IPV6 => 229,
            DataLink::IEEE802_15_4_NOFCS => 230,
            DataLink::DBUS => 231,
            DataLink::DVB_CI => 235,
            DataLink::MUX27010 => 236,
            DataLink::STANAG_5066_D_PDU => 237,
            DataLink::NFLOG => 239,
            DataLink::NETANALYZER => 240,
            DataLink::NETANALYZER_TRANSPARENT => 241,
            DataLink::IPOIB => 242,
            DataLink::MPEG_2_TS => 243,
            DataLink::NG40 => 244,
            DataLink::NFC_LLCP => 245,
            DataLink::INFINIBAND => 247,
            DataLink::SCTP => 248,
            DataLink::USBPCAP => 249,
            DataLink::RTAC_SERIAL => 250,
            DataLink::BLUETOOTH_LE_LL => 251,
            DataLink::NETLINK => 253,
            DataLink::BLUETOOTH_LINUX_MONITOR => 254,
            DataLink::BLUETOOTH_BREDR_BB => 255,
            DataLink::BLUETOOTH_LE_LL_WITH_PHDR => 256,
            DataLink::PROFIBUS_DL => 257,
            DataLink::PKTAP => 258,
            DataLink::EPON => 259,
            DataLink::IPMI_HPM_2 => 260,
            DataLink::ZWAVE_R1_R2 => 261,
            DataLink::ZWAVE_R3 => 262,
            DataLink::WATTSTOPPER_DLM => 263,
            DataLink::ISO_14443 => 264,
            DataLink::RDS => 265,
            DataLink::USB_DARWIN => 266,
            DataLink::SDLC => 268,

            DataLink::Unknown(n) => n
        }
    }
}