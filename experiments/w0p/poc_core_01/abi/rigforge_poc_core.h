/*
 * RigForge POC-CORE-01 research ABI
 *
 * RESEARCH ONLY / W0-P / NON-PRODUCTION
 * POC ONLY — NOT CANONICAL
 *
 * This header is the single C ABI used by both the C++ and Rust
 * implementations. It is not a future public RigForge Domain API.
 */

#ifndef RIGFORGE_POC_CORE_H
#define RIGFORGE_POC_CORE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#if defined(_WIN32)
#  if defined(RF_POC_EXPORT)
#    define RF_POC_API __declspec(dllexport)
#  else
#    define RF_POC_API __declspec(dllimport)
#  endif
#else
#  define RF_POC_API
#endif

typedef struct RF_PocAsset RF_PocAsset;

typedef int32_t rf_poc_status;

#define RF_POC_OK                0
#define RF_POC_ERR_NULL         -1
#define RF_POC_ERR_INVALID_INDEX -2
#define RF_POC_ERR_IO           -3
#define RF_POC_ERR_PARSE        -4
#define RF_POC_ERR_BUFFER       -5
#define RF_POC_ERR_INVALID_HANDLE -6

#define RF_POC_SEV_INFO  0
#define RF_POC_SEV_WARN  1
#define RF_POC_SEV_ERROR 2

/*
 * Strings: caller buffer, UTF-8, optional NUL if buf_len allows.
 * On RF_POC_OK, *out_len is bytes written excluding NUL.
 * On RF_POC_ERR_BUFFER, *out_len is required bytes excluding NUL.
 * buf may be NULL to query required size.
 */

RF_POC_API RF_PocAsset *rf_poc_load(const char *path, rf_poc_status *out_status);
RF_POC_API void rf_poc_destroy(RF_PocAsset *asset);

RF_POC_API rf_poc_status rf_poc_asset_id(
    RF_PocAsset *asset,
    char *buf, size_t buf_len, size_t *out_len);

RF_POC_API int32_t rf_poc_joint_count(RF_PocAsset *asset);
RF_POC_API rf_poc_status rf_poc_joint_parent(
    RF_PocAsset *asset, int32_t index, int32_t *out_parent);
RF_POC_API rf_poc_status rf_poc_joint_name(
    RF_PocAsset *asset, int32_t index,
    char *buf, size_t buf_len, size_t *out_len);
RF_POC_API rf_poc_status rf_poc_joint_rest_translation(
    RF_PocAsset *asset, int32_t index, double out_xyz[3]);
RF_POC_API rf_poc_status rf_poc_joint_rest_rotation(
    RF_PocAsset *asset, int32_t index, double out_xyzw[4]);
RF_POC_API rf_poc_status rf_poc_joint_rest_scale(
    RF_PocAsset *asset, int32_t index, double out_xyz[3]);

RF_POC_API int32_t rf_poc_motion_count(RF_PocAsset *asset);
RF_POC_API rf_poc_status rf_poc_motion_name(
    RF_PocAsset *asset, int32_t motion_index,
    char *buf, size_t buf_len, size_t *out_len);
RF_POC_API rf_poc_status rf_poc_motion_time_range(
    RF_PocAsset *asset, int32_t motion_index,
    double *out_t0, double *out_t1);

RF_POC_API int32_t rf_poc_track_count(RF_PocAsset *asset, int32_t motion_index);
RF_POC_API rf_poc_status rf_poc_track_joint(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index,
    int32_t *out_joint);
RF_POC_API int32_t rf_poc_track_translation_key_count(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index);
RF_POC_API int32_t rf_poc_track_rotation_key_count(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index);
RF_POC_API rf_poc_status rf_poc_track_translation_key(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index, int32_t key_index,
    double *out_time, double out_xyz[3]);
RF_POC_API rf_poc_status rf_poc_track_rotation_key(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index, int32_t key_index,
    double *out_time, double out_xyzw[4]);

RF_POC_API int32_t rf_poc_diagnostic_count(RF_PocAsset *asset);
RF_POC_API rf_poc_status rf_poc_diagnostic(
    RF_PocAsset *asset, int32_t index,
    int32_t *out_severity,
    char *code_buf, size_t code_len, size_t *code_out,
    char *msg_buf, size_t msg_len, size_t *msg_out,
    char *loc_buf, size_t loc_len, size_t *loc_out);

#ifdef __cplusplus
}
#endif

#endif /* RIGFORGE_POC_CORE_H */
