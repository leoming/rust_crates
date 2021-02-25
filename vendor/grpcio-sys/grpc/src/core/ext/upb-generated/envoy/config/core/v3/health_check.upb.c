/* This file was generated by upbc (the upb compiler) from the input
 * file:
 *
 *     envoy/config/core/v3/health_check.proto
 *
 * Do not edit -- your changes will be discarded when the file is
 * regenerated. */

#include <stddef.h>
#include "upb/msg.h"
#include "envoy/config/core/v3/health_check.upb.h"
#include "envoy/config/core/v3/base.upb.h"
#include "envoy/config/core/v3/event_service_config.upb.h"
#include "envoy/type/matcher/v3/string.upb.h"
#include "envoy/type/v3/http.upb.h"
#include "envoy/type/v3/range.upb.h"
#include "google/protobuf/any.upb.h"
#include "google/protobuf/duration.upb.h"
#include "google/protobuf/struct.upb.h"
#include "google/protobuf/wrappers.upb.h"
#include "envoy/annotations/deprecation.upb.h"
#include "udpa/annotations/status.upb.h"
#include "udpa/annotations/versioning.upb.h"
#include "validate/validate.upb.h"

#include "upb/port_def.inc"

static const upb_msglayout *const envoy_config_core_v3_HealthCheck_submsgs[19] = {
  &envoy_config_core_v3_EventServiceConfig_msginit,
  &envoy_config_core_v3_HealthCheck_CustomHealthCheck_msginit,
  &envoy_config_core_v3_HealthCheck_GrpcHealthCheck_msginit,
  &envoy_config_core_v3_HealthCheck_HttpHealthCheck_msginit,
  &envoy_config_core_v3_HealthCheck_TcpHealthCheck_msginit,
  &envoy_config_core_v3_HealthCheck_TlsOptions_msginit,
  &google_protobuf_BoolValue_msginit,
  &google_protobuf_Duration_msginit,
  &google_protobuf_Struct_msginit,
  &google_protobuf_UInt32Value_msginit,
};

static const upb_msglayout_field envoy_config_core_v3_HealthCheck__fields[22] = {
  {1, UPB_SIZE(16, 24), 0, 7, 11, 1},
  {2, UPB_SIZE(20, 32), 0, 7, 11, 1},
  {3, UPB_SIZE(24, 40), 0, 7, 11, 1},
  {4, UPB_SIZE(28, 48), 0, 9, 11, 1},
  {5, UPB_SIZE(32, 56), 0, 9, 11, 1},
  {6, UPB_SIZE(36, 64), 0, 9, 11, 1},
  {7, UPB_SIZE(40, 72), 0, 6, 11, 1},
  {8, UPB_SIZE(76, 144), UPB_SIZE(-81, -153), 3, 11, 1},
  {9, UPB_SIZE(76, 144), UPB_SIZE(-81, -153), 4, 11, 1},
  {11, UPB_SIZE(76, 144), UPB_SIZE(-81, -153), 2, 11, 1},
  {12, UPB_SIZE(44, 80), 0, 7, 11, 1},
  {13, UPB_SIZE(76, 144), UPB_SIZE(-81, -153), 1, 11, 1},
  {14, UPB_SIZE(48, 88), 0, 7, 11, 1},
  {15, UPB_SIZE(52, 96), 0, 7, 11, 1},
  {16, UPB_SIZE(56, 104), 0, 7, 11, 1},
  {17, UPB_SIZE(8, 8), 0, 0, 9, 1},
  {18, UPB_SIZE(0, 0), 0, 0, 13, 1},
  {19, UPB_SIZE(4, 4), 0, 0, 8, 1},
  {20, UPB_SIZE(60, 112), 0, 7, 11, 1},
  {21, UPB_SIZE(64, 120), 0, 5, 11, 1},
  {22, UPB_SIZE(68, 128), 0, 0, 11, 1},
  {23, UPB_SIZE(72, 136), 0, 8, 11, 1},
};

const upb_msglayout envoy_config_core_v3_HealthCheck_msginit = {
  &envoy_config_core_v3_HealthCheck_submsgs[0],
  &envoy_config_core_v3_HealthCheck__fields[0],
  UPB_SIZE(88, 160), 22, false,
};

static const upb_msglayout_field envoy_config_core_v3_HealthCheck_Payload__fields[2] = {
  {1, UPB_SIZE(0, 0), UPB_SIZE(-9, -17), 0, 9, 1},
  {2, UPB_SIZE(0, 0), UPB_SIZE(-9, -17), 0, 12, 1},
};

const upb_msglayout envoy_config_core_v3_HealthCheck_Payload_msginit = {
  NULL,
  &envoy_config_core_v3_HealthCheck_Payload__fields[0],
  UPB_SIZE(16, 32), 2, false,
};

static const upb_msglayout *const envoy_config_core_v3_HealthCheck_HttpHealthCheck_submsgs[5] = {
  &envoy_config_core_v3_HeaderValueOption_msginit,
  &envoy_config_core_v3_HealthCheck_Payload_msginit,
  &envoy_type_matcher_v3_StringMatcher_msginit,
  &envoy_type_v3_Int64Range_msginit,
};

static const upb_msglayout_field envoy_config_core_v3_HealthCheck_HttpHealthCheck__fields[9] = {
  {1, UPB_SIZE(8, 8), 0, 0, 9, 1},
  {2, UPB_SIZE(16, 24), 0, 0, 9, 1},
  {3, UPB_SIZE(24, 40), 0, 1, 11, 1},
  {4, UPB_SIZE(28, 48), 0, 1, 11, 1},
  {6, UPB_SIZE(36, 64), 0, 0, 11, 3},
  {8, UPB_SIZE(40, 72), 0, 0, 9, 3},
  {9, UPB_SIZE(44, 80), 0, 3, 11, 3},
  {10, UPB_SIZE(0, 0), 0, 0, 14, 1},
  {11, UPB_SIZE(32, 56), 0, 2, 11, 1},
};

const upb_msglayout envoy_config_core_v3_HealthCheck_HttpHealthCheck_msginit = {
  &envoy_config_core_v3_HealthCheck_HttpHealthCheck_submsgs[0],
  &envoy_config_core_v3_HealthCheck_HttpHealthCheck__fields[0],
  UPB_SIZE(48, 96), 9, false,
};

static const upb_msglayout *const envoy_config_core_v3_HealthCheck_TcpHealthCheck_submsgs[2] = {
  &envoy_config_core_v3_HealthCheck_Payload_msginit,
};

static const upb_msglayout_field envoy_config_core_v3_HealthCheck_TcpHealthCheck__fields[2] = {
  {1, UPB_SIZE(0, 0), 0, 0, 11, 1},
  {2, UPB_SIZE(4, 8), 0, 0, 11, 3},
};

const upb_msglayout envoy_config_core_v3_HealthCheck_TcpHealthCheck_msginit = {
  &envoy_config_core_v3_HealthCheck_TcpHealthCheck_submsgs[0],
  &envoy_config_core_v3_HealthCheck_TcpHealthCheck__fields[0],
  UPB_SIZE(8, 16), 2, false,
};

static const upb_msglayout_field envoy_config_core_v3_HealthCheck_RedisHealthCheck__fields[1] = {
  {1, UPB_SIZE(0, 0), 0, 0, 9, 1},
};

const upb_msglayout envoy_config_core_v3_HealthCheck_RedisHealthCheck_msginit = {
  NULL,
  &envoy_config_core_v3_HealthCheck_RedisHealthCheck__fields[0],
  UPB_SIZE(8, 16), 1, false,
};

static const upb_msglayout_field envoy_config_core_v3_HealthCheck_GrpcHealthCheck__fields[2] = {
  {1, UPB_SIZE(0, 0), 0, 0, 9, 1},
  {2, UPB_SIZE(8, 16), 0, 0, 9, 1},
};

const upb_msglayout envoy_config_core_v3_HealthCheck_GrpcHealthCheck_msginit = {
  NULL,
  &envoy_config_core_v3_HealthCheck_GrpcHealthCheck__fields[0],
  UPB_SIZE(16, 32), 2, false,
};

static const upb_msglayout *const envoy_config_core_v3_HealthCheck_CustomHealthCheck_submsgs[1] = {
  &google_protobuf_Any_msginit,
};

static const upb_msglayout_field envoy_config_core_v3_HealthCheck_CustomHealthCheck__fields[2] = {
  {1, UPB_SIZE(0, 0), 0, 0, 9, 1},
  {3, UPB_SIZE(8, 16), UPB_SIZE(-13, -25), 0, 11, 1},
};

const upb_msglayout envoy_config_core_v3_HealthCheck_CustomHealthCheck_msginit = {
  &envoy_config_core_v3_HealthCheck_CustomHealthCheck_submsgs[0],
  &envoy_config_core_v3_HealthCheck_CustomHealthCheck__fields[0],
  UPB_SIZE(16, 32), 2, false,
};

static const upb_msglayout_field envoy_config_core_v3_HealthCheck_TlsOptions__fields[1] = {
  {1, UPB_SIZE(0, 0), 0, 0, 9, 3},
};

const upb_msglayout envoy_config_core_v3_HealthCheck_TlsOptions_msginit = {
  NULL,
  &envoy_config_core_v3_HealthCheck_TlsOptions__fields[0],
  UPB_SIZE(4, 8), 1, false,
};

#include "upb/port_undef.inc"

