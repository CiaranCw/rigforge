/*
 * RESEARCH ONLY / W0-P / NON-PRODUCTION
 * C accessors over pinned ufbx for the Rust candidate.
 */
#ifndef POC_UFBX_ACCESS_H
#define POC_UFBX_ACCESS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct PocUfbxScene PocUfbxScene;

/* out_kind: 0 ok, 1 IO, 2 parse */
PocUfbxScene *poc_ufbx_load(const char *path, int32_t *out_kind);
void poc_ufbx_free(PocUfbxScene *s);

size_t poc_ufbx_node_count(PocUfbxScene *s);
int poc_ufbx_node_is_root(PocUfbxScene *s, size_t index);
/* NULL buf (or buf_len==0) returns exact name byte length without copying. */
size_t poc_ufbx_node_name(PocUfbxScene *s, size_t index, char *buf, size_t buf_len);
int32_t poc_ufbx_parent_index(PocUfbxScene *s, size_t index);
int poc_ufbx_parent_is_root(PocUfbxScene *s, size_t index);
void poc_ufbx_local_t(PocUfbxScene *s, size_t index, double out_xyz[3]);
void poc_ufbx_local_r(PocUfbxScene *s, size_t index, double out_xyzw[4]);
void poc_ufbx_local_s(PocUfbxScene *s, size_t index, double out_xyz[3]);

#ifdef __cplusplus
}
#endif

#endif
