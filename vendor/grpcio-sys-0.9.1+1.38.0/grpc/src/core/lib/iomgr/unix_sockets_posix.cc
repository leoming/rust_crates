/*
 *
 * Copyright 2016 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */
#include <grpc/support/port_platform.h>

#include "src/core/lib/iomgr/port.h"

#ifdef GRPC_HAVE_UNIX_SOCKET

#include "src/core/lib/iomgr/sockaddr.h"

#include <linux/vm_sockets.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/un.h>

#include "absl/strings/str_cat.h"

#include "src/core/lib/address_utils/parse_address.h"
#include "src/core/lib/iomgr/unix_sockets_posix.h"

#include <grpc/support/alloc.h>
#include <grpc/support/log.h>

#include "src/core/lib/gpr/useful.h"

void grpc_create_socketpair_if_unix(int sv[2]) {
  GPR_ASSERT(socketpair(AF_UNIX, SOCK_STREAM, 0, sv) == 0);
}

grpc_error_handle grpc_resolve_unix_domain_address(
    const char* name, grpc_resolved_addresses** addresses) {
  *addresses = static_cast<grpc_resolved_addresses*>(
      gpr_malloc(sizeof(grpc_resolved_addresses)));
  (*addresses)->naddrs = 1;
  (*addresses)->addrs = static_cast<grpc_resolved_address*>(
      gpr_malloc(sizeof(grpc_resolved_address)));
  return grpc_core::UnixSockaddrPopulate(name, (*addresses)->addrs);
}

grpc_error_handle grpc_resolve_vsock_address(
    const char* name, grpc_resolved_addresses** addresses) {
  *addresses = static_cast<grpc_resolved_addresses*>(
      gpr_malloc(sizeof(grpc_resolved_addresses)));
  (*addresses)->naddrs = 1;
  (*addresses)->addrs = static_cast<grpc_resolved_address*>(
      gpr_malloc(sizeof(grpc_resolved_address)));
  return grpc_core::VsockaddrPopulate(name, (*addresses)->addrs);
}

grpc_error_handle grpc_resolve_unix_abstract_domain_address(
    const absl::string_view name, grpc_resolved_addresses** addresses) {
  *addresses = static_cast<grpc_resolved_addresses*>(
      gpr_malloc(sizeof(grpc_resolved_addresses)));
  (*addresses)->naddrs = 1;
  (*addresses)->addrs = static_cast<grpc_resolved_address*>(
      gpr_malloc(sizeof(grpc_resolved_address)));
  return grpc_core::UnixAbstractSockaddrPopulate(name, (*addresses)->addrs);
}

int grpc_is_unix_socket(const grpc_resolved_address* resolved_addr) {
  const grpc_sockaddr* addr =
      reinterpret_cast<const grpc_sockaddr*>(resolved_addr->addr);
  return addr->sa_family == AF_UNIX;
}

int grpc_is_vsock_socket(const grpc_resolved_address* resolved_addr) {
  const grpc_sockaddr* addr =
      reinterpret_cast<const grpc_sockaddr*>(resolved_addr->addr);
  return addr->sa_family == AF_VSOCK;
}

void grpc_unlink_if_unix_domain_socket(
    const grpc_resolved_address* resolved_addr) {
  const grpc_sockaddr* addr =
      reinterpret_cast<const grpc_sockaddr*>(resolved_addr->addr);

  if (addr->sa_family != AF_UNIX) {
    return;
  }
  struct sockaddr_un* un = reinterpret_cast<struct sockaddr_un*>(
      const_cast<char*>(resolved_addr->addr));

  // There is nothing to unlink for an abstract unix socket
  if (un->sun_path[0] == '\0' && un->sun_path[1] != '\0') {
    return;
  }

  struct stat st;
  if (stat(un->sun_path, &st) == 0 && (st.st_mode & S_IFMT) == S_IFSOCK) {
    unlink(un->sun_path);
  }
}

std::string grpc_sockaddr_to_uri_unix_if_possible(
    const grpc_resolved_address* resolved_addr) {
  const grpc_sockaddr* addr =
      reinterpret_cast<const grpc_sockaddr*>(resolved_addr->addr);
  if (addr->sa_family != AF_UNIX) {
    return "";
  }
  const auto* unix_addr = reinterpret_cast<const struct sockaddr_un*>(addr);
  if (unix_addr->sun_path[0] == '\0' && unix_addr->sun_path[1] != '\0') {
    return absl::StrCat(
        "unix-abstract:",
        absl::string_view(
            unix_addr->sun_path + 1,
            resolved_addr->len - sizeof(unix_addr->sun_family) - 1));
  }
  return absl::StrCat("unix:", unix_addr->sun_path);
}

std::string grpc_sockaddr_to_vsock(const grpc_resolved_address* resolved_addr) {
  const grpc_sockaddr* addr =
      reinterpret_cast<const grpc_sockaddr*>(resolved_addr->addr);
  if (addr->sa_family != AF_VSOCK) {
    return "";
  }
  const auto* vsock_addr = reinterpret_cast<const struct sockaddr_vm*>(addr);
  return absl::StrCat("vsock:", vsock_addr->svm_cid, ":", vsock_addr->svm_port);
}

#endif
