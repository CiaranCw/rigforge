/*
 * Thin C accessors over ufbx structs for the Rust candidate.
 * RESEARCH ONLY / W0-P / NON-PRODUCTION
 *
 * C++ includes ufbx.h directly. Rust cannot portably layout ufbx_node
 * without bindgen/libclang, so this file is the Rust-side FFI tax.
 */

#include "ufbx_access.h"
#include "ufbx.h"

#include <string.h>
#include <stdlib.h>

struct PocUfbxScene {
    ufbx_scene *scene;
};

PocUfbxScene *poc_ufbx_load(const char *path, int32_t *out_kind)
{
    if (out_kind) {
        *out_kind = 0;
    }
    if (!path) {
        if (out_kind) {
            *out_kind = 1;
        }
        return NULL;
    }
    ufbx_load_opts opts;
    memset(&opts, 0, sizeof(opts));
    ufbx_error error;
    memset(&error, 0, sizeof(error));
    ufbx_scene *scene = ufbx_load_file(path, &opts, &error);
    if (!scene) {
        if (out_kind) {
            *out_kind = (error.type == UFBX_ERROR_FILE_NOT_FOUND) ? 1 : 2;
        }
        return NULL;
    }
    PocUfbxScene *wrap = (PocUfbxScene *)malloc(sizeof(PocUfbxScene));
    if (!wrap) {
        ufbx_free_scene(scene);
        if (out_kind) {
            *out_kind = 2;
        }
        return NULL;
    }
    wrap->scene = scene;
    return wrap;
}

void poc_ufbx_free(PocUfbxScene *s)
{
    if (!s) {
        return;
    }
    if (s->scene) {
        ufbx_free_scene(s->scene);
    }
    free(s);
}

size_t poc_ufbx_node_count(PocUfbxScene *s)
{
    if (!s || !s->scene) {
        return 0;
    }
    return s->scene->nodes.count;
}

static ufbx_node *node_at(PocUfbxScene *s, size_t index)
{
    if (!s || !s->scene || index >= s->scene->nodes.count) {
        return NULL;
    }
    return s->scene->nodes.data[index];
}

int poc_ufbx_node_is_root(PocUfbxScene *s, size_t index)
{
    ufbx_node *n = node_at(s, index);
    return n && n->is_root ? 1 : 0;
}

size_t poc_ufbx_node_name(PocUfbxScene *s, size_t index, char *buf, size_t buf_len)
{
    ufbx_node *n = node_at(s, index);
    if (!n || !n->name.data) {
        if (buf && buf_len > 0) {
            buf[0] = '\0';
        }
        return 0;
    }
    size_t nlen = n->name.length;
    if (buf && buf_len > 0) {
        size_t copy = nlen;
        if (copy >= buf_len) {
            copy = buf_len - 1;
        }
        memcpy(buf, n->name.data, copy);
        buf[copy] = '\0';
    }
    /* buf == NULL: return nlen only (two-pass query). */
    return nlen;
}

int32_t poc_ufbx_parent_index(PocUfbxScene *s, size_t index)
{
    ufbx_node *n = node_at(s, index);
    if (!n || !n->parent || !s || !s->scene) {
        return -1;
    }
    for (size_t i = 0; i < s->scene->nodes.count; ++i) {
        if (s->scene->nodes.data[i] == n->parent) {
            return (int32_t)i;
        }
    }
    return -1;
}

void poc_ufbx_local_t(PocUfbxScene *s, size_t index, double out_xyz[3])
{
    ufbx_node *n = node_at(s, index);
    if (!n || !out_xyz) {
        return;
    }
    out_xyz[0] = n->local_transform.translation.x;
    out_xyz[1] = n->local_transform.translation.y;
    out_xyz[2] = n->local_transform.translation.z;
}

void poc_ufbx_local_r(PocUfbxScene *s, size_t index, double out_xyzw[4])
{
    ufbx_node *n = node_at(s, index);
    if (!n || !out_xyzw) {
        return;
    }
    out_xyzw[0] = n->local_transform.rotation.x;
    out_xyzw[1] = n->local_transform.rotation.y;
    out_xyzw[2] = n->local_transform.rotation.z;
    out_xyzw[3] = n->local_transform.rotation.w;
}

void poc_ufbx_local_s(PocUfbxScene *s, size_t index, double out_xyz[3])
{
    ufbx_node *n = node_at(s, index);
    if (!n || !out_xyz) {
        return;
    }
    out_xyz[0] = n->local_transform.scale.x;
    out_xyz[1] = n->local_transform.scale.y;
    out_xyz[2] = n->local_transform.scale.z;
}

int poc_ufbx_parent_is_root(PocUfbxScene *s, size_t index)
{
    ufbx_node *n = node_at(s, index);
    if (!n || !n->parent) {
        return 1;
    }
    return n->parent->is_root ? 1 : 0;
}
