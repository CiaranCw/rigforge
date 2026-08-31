/* POC-FBX-01 research inspection harness.
 *
 * RESEARCH ONLY / W0-P / NON-PRODUCTION.
 *
 * C99 is the test harness language only.
 * This is not evidence selecting the RigForge Core language.
 *
 * Test subject: ufbx C API (pinned v0.23.0).
 * Load options are zeroed: no invisible axis/unit normalization.
 */

#include <ctype.h>
#include <math.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif
#include <windows.h>
#include <psapi.h>
#endif

#include "ufbx.h"

#define MAX_SAMPLE_NAMES 16
#define MAX_ANIM_PROP_DUMP 200
#define JSON_STACK 64

typedef struct {
	FILE *f;
	int depth;
	int count[JSON_STACK];
} Json;

static void j_nl(Json *j)
{
	fputc('\n', j->f);
	for (int i = 0; i < j->depth; i++)
		fputs("  ", j->f);
}

static void j_sep(Json *j)
{
	if (j->count[j->depth]++ > 0)
		fputc(',', j->f);
	j_nl(j);
}

static void j_raw_key(Json *j, const char *key)
{
	j_sep(j);
	fprintf(j->f, "\"%s\": ", key);
}

static void j_begin_obj(Json *j)
{
	fputc('{', j->f);
	j->depth++;
	j->count[j->depth] = 0;
}

static void j_begin_arr(Json *j)
{
	fputc('[', j->f);
	j->depth++;
	j->count[j->depth] = 0;
}

static void j_end_obj(Json *j)
{
	int n = j->count[j->depth];
	j->depth--;
	if (n > 0)
		j_nl(j);
	fputc('}', j->f);
}

static void j_end_arr(Json *j)
{
	int n = j->count[j->depth];
	j->depth--;
	if (n > 0)
		j_nl(j);
	fputc(']', j->f);
}

static void j_key_obj(Json *j, const char *key)
{
	j_raw_key(j, key);
	j_begin_obj(j);
}

static void j_key_arr(Json *j, const char *key)
{
	j_raw_key(j, key);
	j_begin_arr(j);
}

static void json_escape(FILE *f, const char *s, size_t n)
{
	fputc('"', f);
	for (size_t i = 0; i < n; i++) {
		unsigned char c = (unsigned char)s[i];
		if (c == '"' || c == '\\') {
			fputc('\\', f);
			fputc((char)c, f);
		} else if (c == '\n') {
			fputs("\\n", f);
		} else if (c == '\r') {
			fputs("\\r", f);
		} else if (c == '\t') {
			fputs("\\t", f);
		} else if (c < 0x20) {
			fprintf(f, "\\u%04x", c);
		} else {
			fputc((char)c, f);
		}
	}
	fputc('"', f);
}

static void j_str_n(Json *j, const char *s, size_t n)
{
	json_escape(j->f, s ? s : "", n);
}

static void j_str(Json *j, const char *s)
{
	j_str_n(j, s, s ? strlen(s) : 0);
}

static void j_ustr(Json *j, ufbx_string s)
{
	j_str_n(j, s.data, s.length);
}

static void j_key_str(Json *j, const char *key, const char *s)
{
	j_raw_key(j, key);
	j_str(j, s);
}

static void j_key_ustr(Json *j, const char *key, ufbx_string s)
{
	j_raw_key(j, key);
	j_ustr(j, s);
}

static void j_key_bool(Json *j, const char *key, int v)
{
	j_raw_key(j, key);
	fputs(v ? "true" : "false", j->f);
}

static void j_key_null(Json *j, const char *key)
{
	j_raw_key(j, key);
	fputs("null", j->f);
}

static void j_key_i64(Json *j, const char *key, int64_t v)
{
	j_raw_key(j, key);
	fprintf(j->f, "%lld", (long long)v);
}

static void j_key_u64(Json *j, const char *key, uint64_t v)
{
	j_raw_key(j, key);
	fprintf(j->f, "%llu", (unsigned long long)v);
}

static void j_key_sz(Json *j, const char *key, size_t v)
{
	j_raw_key(j, key);
	fprintf(j->f, "%zu", v);
}

static void j_key_f(Json *j, const char *key, double v)
{
	j_raw_key(j, key);
	if (v != v || v > 1e300 || v < -1e300)
		fputs("null", j->f);
	else
		fprintf(j->f, "%.17g", v);
}

static void j_vec3(Json *j, ufbx_vec3 v)
{
	j_begin_arr(j);
	j_sep(j); fprintf(j->f, "%.17g", (double)v.x);
	j_sep(j); fprintf(j->f, "%.17g", (double)v.y);
	j_sep(j); fprintf(j->f, "%.17g", (double)v.z);
	j_end_arr(j);
}

static void j_key_vec3(Json *j, const char *key, ufbx_vec3 v)
{
	j_raw_key(j, key);
	j_vec3(j, v);
}

static void j_quat(Json *j, ufbx_quat v)
{
	j_begin_obj(j);
	j_key_f(j, "x", v.x);
	j_key_f(j, "y", v.y);
	j_key_f(j, "z", v.z);
	j_key_f(j, "w", v.w);
	j_end_obj(j);
}

static void j_key_quat(Json *j, const char *key, ufbx_quat v)
{
	j_raw_key(j, key);
	j_quat(j, v);
}

static void j_matrix(Json *j, ufbx_matrix m)
{
	j_begin_obj(j);
	j_key_str(j, "layout", "ufbx_matrix.cols[4] as XYZ basis + translation");
	j_key_arr(j, "cols");
	for (int c = 0; c < 4; c++) {
		j_sep(j);
		j_vec3(j, m.cols[c]);
	}
	j_end_arr(j);
	j_end_obj(j);
}

static void j_key_matrix(Json *j, const char *key, ufbx_matrix m)
{
	j_raw_key(j, key);
	j_matrix(j, m);
}

static void j_transform(Json *j, ufbx_transform t)
{
	j_begin_obj(j);
	j_key_vec3(j, "translation", t.translation);
	j_key_quat(j, "rotation_quat", t.rotation);
	j_key_vec3(j, "scale", t.scale);
	j_end_obj(j);
}

static void j_key_transform(Json *j, const char *key, ufbx_transform t)
{
	j_raw_key(j, key);
	j_transform(j, t);
}

/* --- SHA-256 (compact, public-domain style) --- */

typedef struct {
	uint32_t state[8];
	uint64_t bitcount;
	uint8_t buffer[64];
	size_t buffer_len;
} Sha256;

static uint32_t rotr32(uint32_t x, int n) { return (x >> n) | (x << (32 - n)); }

static void sha256_init(Sha256 *s)
{
	s->state[0] = 0x6a09e667u; s->state[1] = 0xbb67ae85u;
	s->state[2] = 0x3c6ef372u; s->state[3] = 0xa54ff53au;
	s->state[4] = 0x510e527fu; s->state[5] = 0x9b05688cu;
	s->state[6] = 0x1f83d9abu; s->state[7] = 0x5be0cd19u;
	s->bitcount = 0;
	s->buffer_len = 0;
}

static void sha256_block(Sha256 *s, const uint8_t blk[64])
{
	static const uint32_t K[64] = {
		0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
		0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
		0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
		0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
		0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
		0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
		0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
		0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2
	};
	uint32_t w[64];
	for (int i = 0; i < 16; i++) {
		w[i] = ((uint32_t)blk[i * 4] << 24) | ((uint32_t)blk[i * 4 + 1] << 16) |
			((uint32_t)blk[i * 4 + 2] << 8) | (uint32_t)blk[i * 4 + 3];
	}
	for (int i = 16; i < 64; i++) {
		uint32_t s0 = rotr32(w[i - 15], 7) ^ rotr32(w[i - 15], 18) ^ (w[i - 15] >> 3);
		uint32_t s1 = rotr32(w[i - 2], 17) ^ rotr32(w[i - 2], 19) ^ (w[i - 2] >> 10);
		w[i] = w[i - 16] + s0 + w[i - 7] + s1;
	}
	uint32_t a = s->state[0], b = s->state[1], c = s->state[2], d = s->state[3];
	uint32_t e = s->state[4], f = s->state[5], g = s->state[6], h = s->state[7];
	for (int i = 0; i < 64; i++) {
		uint32_t S1 = rotr32(e, 6) ^ rotr32(e, 11) ^ rotr32(e, 25);
		uint32_t ch = (e & f) ^ ((~e) & g);
		uint32_t t1 = h + S1 + ch + K[i] + w[i];
		uint32_t S0 = rotr32(a, 2) ^ rotr32(a, 13) ^ rotr32(a, 22);
		uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
		uint32_t t2 = S0 + maj;
		h = g; g = f; f = e; e = d + t1;
		d = c; c = b; b = a; a = t1 + t2;
	}
	s->state[0] += a; s->state[1] += b; s->state[2] += c; s->state[3] += d;
	s->state[4] += e; s->state[5] += f; s->state[6] += g; s->state[7] += h;
}

static void sha256_update(Sha256 *s, const void *data, size_t n)
{
	const uint8_t *p = (const uint8_t *)data;
	s->bitcount += (uint64_t)n * 8;
	while (n) {
		size_t room = 64 - s->buffer_len;
		size_t take = n < room ? n : room;
		memcpy(s->buffer + s->buffer_len, p, take);
		s->buffer_len += take;
		p += take;
		n -= take;
		if (s->buffer_len == 64) {
			sha256_block(s, s->buffer);
			s->buffer_len = 0;
		}
	}
}

static void sha256_final(Sha256 *s, uint8_t out[32])
{
	uint64_t bits = s->bitcount;
	uint8_t block[64];
	memset(block, 0, sizeof block);
	memcpy(block, s->buffer, s->buffer_len);
	size_t i = s->buffer_len;
	block[i++] = 0x80;
	if (i > 56) {
		while (i < 64)
			block[i++] = 0;
		sha256_block(s, block);
		memset(block, 0, 64);
		i = 0;
	}
	while (i < 56)
		block[i++] = 0;
	for (int k = 0; k < 8; k++)
		block[56 + k] = (uint8_t)(bits >> ((7 - k) * 8));
	sha256_block(s, block);
	for (int b = 0; b < 8; b++) {
		out[b * 4] = (uint8_t)(s->state[b] >> 24);
		out[b * 4 + 1] = (uint8_t)(s->state[b] >> 16);
		out[b * 4 + 2] = (uint8_t)(s->state[b] >> 8);
		out[b * 4 + 3] = (uint8_t)s->state[b];
	}
}

static int sha256_file(const char *path, char hex[65], uint64_t *size_out)
{
	FILE *fp = fopen(path, "rb");
	if (!fp)
		return 0;
	Sha256 s;
	sha256_init(&s);
	uint8_t buf[8192];
	uint64_t total = 0;
	size_t n;
	while ((n = fread(buf, 1, sizeof buf, fp)) > 0) {
		sha256_update(&s, buf, n);
		total += n;
	}
	fclose(fp);
	uint8_t digest[32];
	sha256_final(&s, digest);
	for (int i = 0; i < 32; i++)
		sprintf(hex + i * 2, "%02x", digest[i]);
	hex[64] = 0;
	if (size_out)
		*size_out = total;
	return 1;
}

static const char *axis_name(ufbx_coordinate_axis a)
{
	switch (a) {
	case UFBX_COORDINATE_AXIS_POSITIVE_X: return "+x";
	case UFBX_COORDINATE_AXIS_NEGATIVE_X: return "-x";
	case UFBX_COORDINATE_AXIS_POSITIVE_Y: return "+y";
	case UFBX_COORDINATE_AXIS_NEGATIVE_Y: return "-y";
	case UFBX_COORDINATE_AXIS_POSITIVE_Z: return "+z";
	case UFBX_COORDINATE_AXIS_NEGATIVE_Z: return "-z";
	default: return "unknown";
	}
}

static const char *elem_type_name(ufbx_element_type t)
{
	switch (t) {
	case UFBX_ELEMENT_UNKNOWN: return "unknown";
	case UFBX_ELEMENT_NODE: return "node";
	case UFBX_ELEMENT_MESH: return "mesh";
	case UFBX_ELEMENT_LIGHT: return "light";
	case UFBX_ELEMENT_CAMERA: return "camera";
	case UFBX_ELEMENT_BONE: return "bone";
	case UFBX_ELEMENT_EMPTY: return "empty";
	case UFBX_ELEMENT_LINE_CURVE: return "line_curve";
	case UFBX_ELEMENT_NURBS_CURVE: return "nurbs_curve";
	case UFBX_ELEMENT_NURBS_SURFACE: return "nurbs_surface";
	case UFBX_ELEMENT_SKIN_DEFORMER: return "skin_deformer";
	case UFBX_ELEMENT_SKIN_CLUSTER: return "skin_cluster";
	case UFBX_ELEMENT_BLEND_DEFORMER: return "blend_deformer";
	case UFBX_ELEMENT_BLEND_CHANNEL: return "blend_channel";
	case UFBX_ELEMENT_BLEND_SHAPE: return "blend_shape";
	case UFBX_ELEMENT_CACHE_DEFORMER: return "cache_deformer";
	case UFBX_ELEMENT_ANIM_STACK: return "anim_stack";
	case UFBX_ELEMENT_ANIM_LAYER: return "anim_layer";
	case UFBX_ELEMENT_ANIM_VALUE: return "anim_value";
	case UFBX_ELEMENT_ANIM_CURVE: return "anim_curve";
	case UFBX_ELEMENT_CHARACTER: return "character";
	case UFBX_ELEMENT_CONSTRAINT: return "constraint";
	case UFBX_ELEMENT_POSE: return "pose";
	case UFBX_ELEMENT_LOD_GROUP: return "lod_group";
	case UFBX_ELEMENT_MARKER: return "marker";
	default: return "other";
	}
}

static const char *inherit_name(ufbx_inherit_mode m)
{
	switch (m) {
	case UFBX_INHERIT_MODE_NORMAL: return "normal";
	case UFBX_INHERIT_MODE_IGNORE_PARENT_SCALE: return "ignore_parent_scale";
	case UFBX_INHERIT_MODE_COMPONENTWISE_SCALE: return "componentwise_scale";
	default: return "unknown";
	}
}

static const char *rot_order_name(ufbx_rotation_order o)
{
	switch (o) {
	case UFBX_ROTATION_ORDER_XYZ: return "xyz";
	case UFBX_ROTATION_ORDER_XZY: return "xzy";
	case UFBX_ROTATION_ORDER_YZX: return "yzx";
	case UFBX_ROTATION_ORDER_YXZ: return "yxz";
	case UFBX_ROTATION_ORDER_ZXY: return "zxy";
	case UFBX_ROTATION_ORDER_ZYX: return "zyx";
	case UFBX_ROTATION_ORDER_SPHERIC: return "spheric";
	default: return "unknown";
	}
}

static const char *exporter_name(ufbx_exporter e)
{
	switch (e) {
	case UFBX_EXPORTER_UNKNOWN: return "unknown";
	case UFBX_EXPORTER_FBX_SDK: return "fbx_sdk";
	case UFBX_EXPORTER_BLENDER_BINARY: return "blender_binary";
	case UFBX_EXPORTER_BLENDER_ASCII: return "blender_ascii";
	case UFBX_EXPORTER_MOTION_BUILDER: return "motion_builder";
	case UFBX_EXPORTER_UFBX_WRITE: return "ufbx_write";
	default: return "other";
	}
}

static const char *file_format_name(ufbx_file_format f)
{
	switch (f) {
	case UFBX_FILE_FORMAT_FBX: return "fbx";
	case UFBX_FILE_FORMAT_OBJ: return "obj";
	case UFBX_FILE_FORMAT_MTL: return "mtl";
	default: return "unknown";
	}
}

static const char *warning_name(ufbx_warning_type t)
{
	switch (t) {
	case UFBX_WARNING_MISSING_EXTERNAL_FILE: return "MISSING_EXTERNAL_FILE";
	case UFBX_WARNING_IMPLICIT_MTL: return "IMPLICIT_MTL";
	case UFBX_WARNING_TRUNCATED_ARRAY: return "TRUNCATED_ARRAY";
	case UFBX_WARNING_MISSING_GEOMETRY_DATA: return "MISSING_GEOMETRY_DATA";
	case UFBX_WARNING_DUPLICATE_CONNECTION: return "DUPLICATE_CONNECTION";
	case UFBX_WARNING_BAD_VERTEX_W_ATTRIBUTE: return "BAD_VERTEX_W_ATTRIBUTE";
	case UFBX_WARNING_MISSING_POLYGON_MAPPING: return "MISSING_POLYGON_MAPPING";
	case UFBX_WARNING_UNSUPPORTED_VERSION: return "UNSUPPORTED_VERSION";
	case UFBX_WARNING_INDEX_CLAMPED: return "INDEX_CLAMPED";
	case UFBX_WARNING_BAD_UNICODE: return "BAD_UNICODE";
	case UFBX_WARNING_BAD_BASE64_CONTENT: return "BAD_BASE64_CONTENT";
	case UFBX_WARNING_BAD_ELEMENT_CONNECTED_TO_ROOT: return "BAD_ELEMENT_CONNECTED_TO_ROOT";
	case UFBX_WARNING_DUPLICATE_OBJECT_ID: return "DUPLICATE_OBJECT_ID";
	case UFBX_WARNING_EMPTY_FACE_REMOVED: return "EMPTY_FACE_REMOVED";
	case UFBX_WARNING_UNKNOWN_OBJ_DIRECTIVE: return "UNKNOWN_OBJ_DIRECTIVE";
	default: return "OTHER";
	}
}

static const char *space_conv_name(ufbx_space_conversion c)
{
	switch (c) {
	case UFBX_SPACE_CONVERSION_TRANSFORM_ROOT: return "transform_root";
	case UFBX_SPACE_CONVERSION_ADJUST_TRANSFORMS: return "adjust_transforms";
	case UFBX_SPACE_CONVERSION_MODIFY_GEOMETRY: return "modify_geometry";
	default: return "unknown";
	}
}

static const char *geo_handling_name(ufbx_geometry_transform_handling h)
{
	switch (h) {
	case UFBX_GEOMETRY_TRANSFORM_HANDLING_PRESERVE: return "preserve";
	case UFBX_GEOMETRY_TRANSFORM_HANDLING_HELPER_NODES: return "helper_nodes";
	case UFBX_GEOMETRY_TRANSFORM_HANDLING_MODIFY_GEOMETRY: return "modify_geometry";
	case UFBX_GEOMETRY_TRANSFORM_HANDLING_MODIFY_GEOMETRY_NO_FALLBACK: return "modify_geometry_no_fallback";
	default: return "unknown";
	}
}

static const char *pivot_handling_name(ufbx_pivot_handling h)
{
	switch (h) {
	case UFBX_PIVOT_HANDLING_RETAIN: return "retain";
	case UFBX_PIVOT_HANDLING_ADJUST_TO_PIVOT: return "adjust_to_pivot";
	case UFBX_PIVOT_HANDLING_ADJUST_TO_ROTATION_PIVOT: return "adjust_to_rotation_pivot";
	default: return "unknown";
	}
}

static const char *inherit_handling_name(ufbx_inherit_mode_handling h)
{
	switch (h) {
	case UFBX_INHERIT_MODE_HANDLING_PRESERVE: return "preserve";
	case UFBX_INHERIT_MODE_HANDLING_HELPER_NODES: return "helper_nodes";
	case UFBX_INHERIT_MODE_HANDLING_COMPENSATE: return "compensate";
	default: return "other";
	}
}

static const char *basename_of(const char *path)
{
	const char *p = path;
	const char *last = path;
	for (; *p; p++) {
		if (*p == '/' || *p == '\\')
			last = p + 1;
	}
	return last;
}

static int ustr_eq(ufbx_string a, const char *b)
{
	size_t n = strlen(b);
	return a.length == n && memcmp(a.data, b, n) == 0;
}

static int node_index(const ufbx_scene *scene, const ufbx_node *node)
{
	if (!node)
		return -1;
	for (size_t i = 0; i < scene->nodes.count; i++) {
		if (scene->nodes.data[i] == node)
			return (int)i;
	}
	return -1;
}

static int prop_found(const ufbx_prop *p)
{
	return p && !(p->flags & UFBX_PROP_FLAG_NOT_FOUND);
}

static void emit_prop_vec3(Json *j, const ufbx_node *node, const char *name)
{
	ufbx_prop *p = ufbx_find_prop(&node->props, name);
	j_raw_key(j, name);
	j_begin_obj(j);
	j_key_bool(j, "found", prop_found(p));
	if (prop_found(p)) {
		ufbx_vec3 v = { p->value_vec4.x, p->value_vec4.y, p->value_vec4.z };
		j_key_vec3(j, "value", v);
		j_key_u64(j, "flags", (uint64_t)p->flags);
		j_key_bool(j, "synthetic", (p->flags & UFBX_PROP_FLAG_SYNTHETIC) != 0);
		j_key_bool(j, "animated", (p->flags & UFBX_PROP_FLAG_ANIMATED) != 0);
	}
	j_end_obj(j);
}

static void emit_authored_props(Json *j, const ufbx_node *node)
{
	j_key_obj(j, "authored_source_props");
	j_key_str(j, "api", "ufbx_find_prop(&node->props, name)");
	j_key_str(j, "note", "RAW SOURCE STRUCTURE. These are FBX property values, not ufbx_node.local_transform.");
	const char *names[] = {
		"Lcl Translation", "Lcl Rotation", "Lcl Scaling",
		"RotationOffset", "RotationPivot", "ScalingOffset", "ScalingPivot",
		"PreRotation", "PostRotation",
		"GeometricTranslation", "GeometricRotation", "GeometricScaling",
		NULL
	};
	for (int i = 0; names[i]; i++)
		emit_prop_vec3(j, node, names[i]);
	{
		ufbx_prop *p = ufbx_find_prop(&node->props, "InheritType");
		j_raw_key(j, "InheritType");
		j_begin_obj(j);
		j_key_bool(j, "found", prop_found(p));
		if (prop_found(p))
			j_key_i64(j, "value_int", p->value_int);
		j_end_obj(j);
	}
	{
		ufbx_prop *p = ufbx_find_prop(&node->props, "RotationActive");
		j_raw_key(j, "RotationActive");
		j_begin_obj(j);
		j_key_bool(j, "found", prop_found(p));
		if (prop_found(p))
			j_key_i64(j, "value_int", p->value_int);
		j_end_obj(j);
	}
	j_end_obj(j);
}

typedef struct {
	char *names[MAX_SAMPLE_NAMES];
	int count;
} SampleNames;

static int want_sample(const SampleNames *sn, ufbx_string name, int auto_idx, const int *auto_set, int auto_n)
{
	if (sn->count > 0) {
		for (int i = 0; i < sn->count; i++) {
			if (ustr_eq(name, sn->names[i]))
				return 1;
		}
		return 0;
	}
	for (int i = 0; i < auto_n; i++) {
		if (auto_set[i] == auto_idx)
			return 1;
	}
	return 0;
}

static void pick_auto_samples(const ufbx_scene *scene, const uint8_t *is_cluster, int out_idx[4], int *out_n)
{
	/* 0 root/trajectory candidate, 1 central_body_candidate, 2 intermediate, 3 leaf.
	 * No humanoid / pelvis assumption. */
	int best_root = -1, best_leaf = -1, best_mid = -1, best_central = -1;
	uint32_t min_depth = UINT32_MAX, max_depth = 0;
	int pool_n = 0;
	int central_score = -1;

	for (size_t i = 0; i < scene->nodes.count; i++) {
		ufbx_node *n = scene->nodes.data[i];
		if (n->is_root)
			continue;
		int interesting = n->bone != NULL || is_cluster[i] || n->attrib_type == UFBX_ELEMENT_BONE;
		if (!interesting)
			continue;
		pool_n++;
		if (n->node_depth < min_depth) {
			min_depth = n->node_depth;
			best_root = (int)i;
		} else if (n->node_depth == min_depth && (best_root < 0 || (int)i < best_root)) {
			best_root = (int)i;
		}
		if (n->node_depth > max_depth) {
			max_depth = n->node_depth;
			best_leaf = (int)i;
		} else if (n->node_depth == max_depth && (best_leaf < 0 || (int)i < best_leaf)) {
			best_leaf = (int)i;
		}
		if (is_cluster[i]) {
			int score = (int)n->children.count;
			if (score > central_score || (score == central_score && (best_central < 0 || (int)i < best_central))) {
				central_score = score;
				best_central = (int)i;
			}
		}
	}

	if (pool_n == 0) {
		for (size_t i = 0; i < scene->nodes.count; i++) {
			if (!scene->nodes.data[i]->is_root) {
				best_root = (int)i;
				break;
			}
		}
	}

	if (best_central < 0)
		best_central = best_root;

	/* median-depth interesting node */
	{
		uint32_t depths[1024];
		int dc = 0;
		for (size_t i = 0; i < scene->nodes.count && dc < 1024; i++) {
			ufbx_node *n = scene->nodes.data[i];
			if (n->is_root)
				continue;
			if (!(n->bone || is_cluster[i] || n->attrib_type == UFBX_ELEMENT_BONE))
				continue;
			depths[dc++] = n->node_depth;
		}
		if (dc > 0) {
			for (int a = 0; a < dc; a++) {
				for (int b = a + 1; b < dc; b++) {
					if (depths[b] < depths[a]) {
						uint32_t t = depths[a]; depths[a] = depths[b]; depths[b] = t;
					}
				}
			}
			uint32_t med = depths[dc / 2];
			for (size_t i = 0; i < scene->nodes.count; i++) {
				ufbx_node *n = scene->nodes.data[i];
				if (n->is_root)
					continue;
				if (!(n->bone || is_cluster[i] || n->attrib_type == UFBX_ELEMENT_BONE))
					continue;
				if (n->node_depth == med) {
					best_mid = (int)i;
					break;
				}
			}
		}
	}

	int n = 0;
	int seen[4];
	int cands[4] = { best_root, best_central, best_mid, best_leaf };
	for (int i = 0; i < 4; i++) {
		if (cands[i] < 0)
			continue;
		int dup = 0;
		for (int k = 0; k < n; k++) {
			if (seen[k] == cands[i])
				dup = 1;
		}
		if (dup)
			continue;
		seen[n] = cands[i];
		out_idx[n++] = cands[i];
	}
	*out_n = n;
}

static const char *auto_role(int slot)
{
	switch (slot) {
	case 0: return "root_or_trajectory_candidate";
	case 1: return "central_body_candidate";
	case 2: return "intermediate_node";
	case 3: return "leaf_or_end_node";
	default: return "sample_subset_member";
	}
}

static int real_finite_nn(ufbx_real w, int *nonfinite, int *negative)
{
	double d = (double)w;
	if (d != d || d > 1e300 || d < -1e300) {
		(*nonfinite)++;
		return 0;
	}
	if (d < 0.0) {
		(*negative)++;
		return 0;
	}
	return 1;
}

static const char *skinning_method_name(ufbx_skinning_method m)
{
	switch (m) {
	case UFBX_SKINNING_METHOD_LINEAR: return "linear";
	case UFBX_SKINNING_METHOD_RIGID: return "rigid";
	case UFBX_SKINNING_METHOD_DUAL_QUATERNION: return "dual_quaternion";
	case UFBX_SKINNING_METHOD_BLENDED_DQ_LINEAR: return "blended_dq_linear";
	default: return "unknown";
	}
}

static void emit_one_vertex_influences(Json *j, ufbx_skin_deformer *skin, size_t vi, const char **roles, int nroles)
{
	ufbx_skin_vertex sv = skin->vertices.data[vi];
	j_begin_obj(j);
	j_key_sz(j, "vertex_index", vi);
	j_key_arr(j, "roles");
	for (int ri = 0; ri < nroles; ri++) {
		j_sep(j);
		j_str(j, roles[ri]);
	}
	j_end_arr(j);
	j_key_u64(j, "num_weights", sv.num_weights);
	j_key_arr(j, "influences");
	for (uint32_t wi = 0; wi < sv.num_weights; wi++) {
		size_t wi_abs = (size_t)sv.weight_begin + (size_t)wi;
		if (wi_abs >= skin->weights.count)
			break;
		ufbx_skin_weight w = skin->weights.data[wi_abs];
		j_sep(j);
		j_begin_obj(j);
		j_key_u64(j, "cluster_index", w.cluster_index);
		if (w.cluster_index < skin->clusters.count) {
			ufbx_skin_cluster *cl = skin->clusters.data[w.cluster_index];
			if (cl->bone_node)
				j_key_ustr(j, "bone_node_name", cl->bone_node->name);
			else
				j_key_null(j, "bone_node_name");
		} else {
			j_key_null(j, "bone_node_name");
		}
		j_key_f(j, "weight", (double)w.weight);
		j_end_obj(j);
	}
	j_end_arr(j);
	j_end_obj(j);
}

static void emit_skin_vertex_weights(Json *j, ufbx_skin_deformer *skin)
{
	size_t nvert = skin->vertices.count;
	size_t nweight = skin->weights.count;
	size_t first_inf = (size_t)-1, last_inf = (size_t)-1, max_inf_vi = (size_t)-1;
	size_t influenced = 0, zero_inf = 0;
	uint32_t min_n = 0xFFFFFFFFu, max_n = 0;
	int invalid_cluster = 0, nonfinite = 0, negative = 0, oob_weight = 0;
	double min_sum = 0, max_sum = 0;
	int have_sum = 0;

	j_key_obj(j, "vertex_weights");
	j_key_str(j, "api", "ufbx_skin_deformer.vertices[] / weights[] (ufbx_skin_vertex, ufbx_skin_weight)");
	j_key_str(j, "ufbx_note", "ufbx documents that per-vertex weights are not guaranteed to be normalized. This PoC does not require sum==1.0.");
	j_key_str(j, "skinning_method", skinning_method_name(skin->skinning_method));
	j_key_sz(j, "skin_vertex_record_count", nvert);
	j_key_sz(j, "total_weight_entry_count", nweight);
	j_key_sz(j, "max_weights_per_vertex_field", skin->max_weights_per_vertex);

	for (size_t vi = 0; vi < nvert; vi++) {
		ufbx_skin_vertex sv = skin->vertices.data[vi];
		if (sv.num_weights == 0) {
			zero_inf++;
			continue;
		}
		if (first_inf == (size_t)-1)
			first_inf = vi;
		last_inf = vi;
		influenced++;
		if (sv.num_weights < min_n)
			min_n = sv.num_weights;
		if (sv.num_weights > max_n) {
			max_n = sv.num_weights;
			max_inf_vi = vi;
		}
		double sum = 0.0;
		for (uint32_t wi = 0; wi < sv.num_weights; wi++) {
			size_t wi_abs = (size_t)sv.weight_begin + (size_t)wi;
			if (wi_abs >= nweight) {
				oob_weight++;
				break;
			}
			ufbx_skin_weight w = skin->weights.data[wi_abs];
			if (w.cluster_index >= skin->clusters.count)
				invalid_cluster++;
			if (real_finite_nn(w.weight, &nonfinite, &negative))
				sum += (double)w.weight;
		}
		if (!have_sum) {
			min_sum = max_sum = sum;
			have_sum = 1;
		} else {
			if (sum < min_sum)
				min_sum = sum;
			if (sum > max_sum)
				max_sum = sum;
		}
	}

	j_key_sz(j, "vertices_with_influences", influenced);
	j_key_sz(j, "zero_influence_vertices", zero_inf);
	if (influenced == 0)
		j_key_null(j, "min_influences_per_vertex");
	else
		j_key_u64(j, "min_influences_per_vertex", min_n);
	j_key_u64(j, "max_influences_per_vertex_observed", max_n);
	if (!have_sum) {
		j_key_null(j, "min_weight_sum");
		j_key_null(j, "max_weight_sum");
	} else {
		j_key_f(j, "min_weight_sum", min_sum);
		j_key_f(j, "max_weight_sum", max_sum);
	}
	j_key_i64(j, "invalid_cluster_refs", invalid_cluster);
	j_key_i64(j, "non_finite_weights", nonfinite);
	j_key_i64(j, "negative_weights", negative);
	j_key_i64(j, "out_of_range_weight_indices", oob_weight);
	j_key_bool(j, "structural_validation_pass",
		invalid_cluster == 0 && nonfinite == 0 && negative == 0 && oob_weight == 0);
	j_key_str(j, "weight_sum_policy", "Observed min/max sums only. Not a requirement that every vertex sums to 1.0.");

	size_t mid_inf = (size_t)-1;
	if (influenced > 0) {
		size_t target = influenced / 2;
		size_t seen = 0;
		for (size_t vi = 0; vi < nvert; vi++) {
			if (skin->vertices.data[vi].num_weights == 0)
				continue;
			if (seen == target) {
				mid_inf = vi;
				break;
			}
			seen++;
		}
	}

	size_t cands[4];
	int nc = 0;
	if (first_inf != (size_t)-1)
		cands[nc++] = first_inf;
	if (mid_inf != (size_t)-1)
		cands[nc++] = mid_inf;
	if (last_inf != (size_t)-1)
		cands[nc++] = last_inf;
	if (max_inf_vi != (size_t)-1)
		cands[nc++] = max_inf_vi;
	for (int a = 0; a < nc; a++) {
		for (int b = a + 1; b < nc; b++) {
			if (cands[b] < cands[a]) {
				size_t t = cands[a];
				cands[a] = cands[b];
				cands[b] = t;
			}
		}
	}
	int nuniq = 0;
	size_t uniq[4];
	for (int i = 0; i < nc; i++) {
		if (nuniq && uniq[nuniq - 1] == cands[i])
			continue;
		uniq[nuniq++] = cands[i];
	}

	j_key_arr(j, "representative_vertices");
	for (int i = 0; i < nuniq; i++) {
		const char *roles[4];
		int nr = 0;
		if (uniq[i] == first_inf)
			roles[nr++] = "first_influenced_vertex";
		if (uniq[i] == mid_inf)
			roles[nr++] = "middle_influenced_vertex";
		if (uniq[i] == last_inf)
			roles[nr++] = "last_influenced_vertex";
		if (uniq[i] == max_inf_vi)
			roles[nr++] = "max_influence_count_vertex";
		j_sep(j);
		emit_one_vertex_influences(j, skin, uniq[i], roles, nr);
	}
	j_end_arr(j);
	j_end_obj(j);
}

int main(int argc, char **argv)
{
	const char *input = NULL;
	const char *out_path = NULL;
	SampleNames sn;
	memset(&sn, 0, sizeof sn);
	int stack_index = 0;

	for (int i = 1; i < argc; i++) {
		if (!strcmp(argv[i], "--input") && i + 1 < argc)
			input = argv[++i];
		else if (!strcmp(argv[i], "--out") && i + 1 < argc)
			out_path = argv[++i];
		else if (!strcmp(argv[i], "--anim-stack-index") && i + 1 < argc)
			stack_index = atoi(argv[++i]);
		else if (!strcmp(argv[i], "--sample-names") && i + 1 < argc) {
			char *tok = strtok(argv[++i], ",");
			while (tok && sn.count < MAX_SAMPLE_NAMES) {
				sn.names[sn.count++] = tok;
				tok = strtok(NULL, ",");
			}
		} else {
			fprintf(stderr, "unknown arg: %s\n", argv[i]);
			return 2;
		}
	}
	if (!input || !out_path) {
		fprintf(stderr, "usage: inspect_fbx --input FILE --out JSON [--sample-names a,b] [--anim-stack-index N]\n");
		return 2;
	}

	char sha_hex[65];
	uint64_t fsize = 0;
	if (!sha256_file(input, sha_hex, &fsize)) {
		fprintf(stderr, "cannot read input: %s\n", input);
		return 1;
	}

#ifdef _WIN32
	LARGE_INTEGER qpf, qpc0, qpc1;
	QueryPerformanceFrequency(&qpf);
	QueryPerformanceCounter(&qpc0);
#endif

	ufbx_load_opts opts;
	memset(&opts, 0, sizeof opts);
	ufbx_error error;
	memset(&error, 0, sizeof error);
	ufbx_scene *scene = ufbx_load_file(input, &opts, &error);

#ifdef _WIN32
	QueryPerformanceCounter(&qpc1);
	double load_ms = (double)(qpc1.QuadPart - qpc0.QuadPart) * 1000.0 / (double)qpf.QuadPart;
	PROCESS_MEMORY_COUNTERS pmc;
	memset(&pmc, 0, sizeof pmc);
	pmc.cb = sizeof pmc;
	SIZE_T peak = 0;
	if (GetProcessMemoryInfo(GetCurrentProcess(), &pmc, sizeof pmc))
		peak = pmc.PeakWorkingSetSize;
	fprintf(stderr, "PERF load_ms=%.3f peak_working_set_bytes=%llu\n",
		load_ms, (unsigned long long)peak);
#else
	fprintf(stderr, "PERF load_ms=null peak_working_set_bytes=null\n");
#endif

	FILE *out = fopen(out_path, "wb");
	if (!out) {
		fprintf(stderr, "cannot write %s\n", out_path);
		if (scene)
			ufbx_free_scene(scene);
		return 1;
	}

	Json j;
	memset(&j, 0, sizeof j);
	j.f = out;
	j.depth = 0;
	j.count[0] = 0;
	j_begin_obj(&j);

	j_key_str(&j, "poc", "POC-FBX-01");
	j_key_str(&j, "classification", "RESEARCH_ONLY / W0-P / NON-PRODUCTION");
	j_key_obj(&j, "harness");
	j_key_str(&j, "language", "C99");
	j_key_str(&j, "note", "C is the test harness language only. This is not evidence selecting the RigForge Core language.");
	j_key_str(&j, "test_subject", "ufbx C API");
	j_key_str(&j, "ufbx_header_version", "0.23.0");
	j_end_obj(&j);

	j_key_obj(&j, "source");
	j_key_str(&j, "basename", basename_of(input));
	j_key_u64(&j, "size_bytes", fsize);
	j_key_str(&j, "sha256", sha_hex);
	j_end_obj(&j);

	j_key_obj(&j, "experiment_selection");
	j_key_i64(&j, "anim_stack_index", stack_index);
	j_key_arr(&j, "sample_names");
	for (int si = 0; si < sn.count; si++) {
		j_sep(&j);
		j_str(&j, sn.names[si]);
	}
	j_end_arr(&j);
	j_key_str(&j, "note", "Explicit research-run selection recorded in JSON. Empty sample_names means harness auto-picker. Not Canonical anatomy.");
	j_end_obj(&j);

	j_key_obj(&j, "load");
	j_key_obj(&j, "opts");
	j_key_bool(&j, "zeroed_ufbx_load_opts", 1);
	j_key_bool(&j, "target_axes_set", 0);
	j_key_f(&j, "target_unit_meters", 0);
	j_key_bool(&j, "ignore_animation", 0);
	j_key_bool(&j, "ignore_geometry", 0);
	j_key_bool(&j, "evaluate_skinning", 0);
	j_key_str(&j, "geometry_transform_handling", "default_zeroed=preserve");
	j_key_str(&j, "inherit_mode_handling", "default_zeroed=preserve");
	j_key_str(&j, "pivot_handling", "default_zeroed=retain");
	j_key_str(&j, "space_conversion", "default_zeroed; inactive unless target_axes/unit set");
	j_key_str(&j, "normalization", "NONE applied by harness. Source axes/units preserved.");
	j_end_obj(&j);

	if (!scene) {
		j_key_bool(&j, "success", 0);
		j_key_ustr(&j, "error_description", error.description);
		j_key_str(&j, "error_info", error.info);
		j_end_obj(&j);
		j_key_null(&j, "derived_test_view");
		j_end_obj(&j);
		fclose(out);
		return 1;
	}

	j_key_bool(&j, "success", 1);
	j_key_arr(&j, "warnings");
	for (size_t i = 0; i < scene->metadata.warnings.count; i++) {
		ufbx_warning *w = &scene->metadata.warnings.data[i];
		j_sep(&j);
		j_begin_obj(&j);
		j_key_str(&j, "type", warning_name(w->type));
		j_key_ustr(&j, "description", w->description);
		j_key_u64(&j, "element_id", w->element_id);
		j_key_sz(&j, "count", w->count);
		j_end_obj(&j);
	}
	j_end_arr(&j);
	j_key_bool(&j, "geometry_ignored", scene->metadata.geometry_ignored);
	j_key_bool(&j, "animation_ignored", scene->metadata.animation_ignored);
	j_key_bool(&j, "embedded_ignored", scene->metadata.embedded_ignored);
	j_end_obj(&j);

	j_key_obj(&j, "scene_metadata");
	j_key_u64(&j, "fbx_version", scene->metadata.version);
	j_key_bool(&j, "ascii", scene->metadata.ascii);
	j_key_str(&j, "file_format", file_format_name(scene->metadata.file_format));
	j_key_ustr(&j, "creator", scene->metadata.creator);
	j_key_str(&j, "exporter", exporter_name(scene->metadata.exporter));
	j_key_u64(&j, "exporter_version", scene->metadata.exporter_version);
	j_key_obj(&j, "original_application");
	j_key_ustr(&j, "vendor", scene->metadata.original_application.vendor);
	j_key_ustr(&j, "name", scene->metadata.original_application.name);
	j_key_ustr(&j, "version", scene->metadata.original_application.version);
	j_end_obj(&j);
	j_key_obj(&j, "latest_application");
	j_key_ustr(&j, "vendor", scene->metadata.latest_application.vendor);
	j_key_ustr(&j, "name", scene->metadata.latest_application.name);
	j_key_ustr(&j, "version", scene->metadata.latest_application.version);
	j_end_obj(&j);
	j_key_str(&j, "space_conversion_applied", space_conv_name(scene->metadata.space_conversion));
	j_key_str(&j, "geometry_transform_handling_applied", geo_handling_name(scene->metadata.geometry_transform_handling));
	j_key_str(&j, "inherit_mode_handling_applied", inherit_handling_name(scene->metadata.inherit_mode_handling));
	j_key_str(&j, "pivot_handling_applied", pivot_handling_name(scene->metadata.pivot_handling));
	j_key_f(&j, "root_scale", scene->metadata.root_scale);
	j_key_f(&j, "geometry_scale", scene->metadata.geometry_scale);
	j_key_i64(&j, "ktime_second", scene->metadata.ktime_second);
	j_end_obj(&j);

	j_key_obj(&j, "coordinate_unit");
	j_key_str(&j, "layer", "RAW SOURCE STRUCTURE / ufbx settings (no harness normalization)");
	j_key_str(&j, "right", axis_name(scene->settings.axes.right));
	j_key_str(&j, "up", axis_name(scene->settings.axes.up));
	j_key_str(&j, "front", axis_name(scene->settings.axes.front));
	j_key_str(&j, "front_note", "ufbx: front is the opposite of forward");
	j_key_f(&j, "unit_meters", scene->settings.unit_meters);
	j_key_f(&j, "original_unit_meters", scene->settings.original_unit_meters);
	j_key_str(&j, "original_axis_up", axis_name(scene->settings.original_axis_up));
	j_key_f(&j, "frames_per_second", scene->settings.frames_per_second);
	j_key_null(&j, "derived_test_view");
	j_key_str(&j, "derived_test_view_note", "Not created. Source facts are not replaced by a normalized view.");
	j_end_obj(&j);

	/* cluster-bone mark */
	uint8_t *is_cluster = (uint8_t *)calloc(scene->nodes.count, 1);
	if (!is_cluster) {
		fclose(out);
		ufbx_free_scene(scene);
		return 1;
	}
	for (size_t si = 0; si < scene->skin_deformers.count; si++) {
		ufbx_skin_deformer *skin = scene->skin_deformers.data[si];
		for (size_t ci = 0; ci < skin->clusters.count; ci++) {
			ufbx_skin_cluster *cl = skin->clusters.data[ci];
			int bi = node_index(scene, cl->bone_node);
			if (bi >= 0)
				is_cluster[bi] = 1;
		}
	}

	j_key_obj(&j, "inventory_counts");
	j_key_sz(&j, "nodes", scene->nodes.count);
	j_key_sz(&j, "meshes", scene->meshes.count);
	j_key_sz(&j, "bones_attrib", scene->bones.count);
	j_key_sz(&j, "empties", scene->empties.count);
	j_key_sz(&j, "skin_deformers", scene->skin_deformers.count);
	j_key_sz(&j, "skin_clusters", scene->skin_clusters.count);
	j_key_sz(&j, "poses", scene->poses.count);
	j_key_sz(&j, "anim_stacks", scene->anim_stacks.count);
	j_key_sz(&j, "anim_layers", scene->anim_layers.count);
	j_key_sz(&j, "anim_curves", scene->anim_curves.count);
	j_key_sz(&j, "characters", scene->characters.count);
	j_key_sz(&j, "constraints", scene->constraints.count);
	j_key_sz(&j, "blend_deformers", scene->blend_deformers.count);
	j_key_sz(&j, "cache_deformers", scene->cache_deformers.count);
	j_key_sz(&j, "nurbs_curves", scene->nurbs_curves.count);
	j_key_sz(&j, "nurbs_surfaces", scene->nurbs_surfaces.count);
	j_end_obj(&j);

	j_key_arr(&j, "root_nodes");
	if (scene->root_node) {
		for (size_t i = 0; i < scene->root_node->children.count; i++) {
			ufbx_node *ch = scene->root_node->children.data[i];
			j_sep(&j);
			j_begin_obj(&j);
			j_key_i64(&j, "index", node_index(scene, ch));
			j_key_ustr(&j, "name", ch->name);
			j_key_str(&j, "attrib_type", elem_type_name(ch->attrib_type));
			j_end_obj(&j);
		}
	}
	j_end_arr(&j);

	int auto_idx[4];
	int auto_n = 0;
	pick_auto_samples(scene, is_cluster, auto_idx, &auto_n);

	j_key_arr(&j, "nodes");
	for (size_t i = 0; i < scene->nodes.count; i++) {
		ufbx_node *n = scene->nodes.data[i];
		j_sep(&j);
		j_begin_obj(&j);
		j_key_sz(&j, "traversal_index", i);
		j_key_u64(&j, "typed_id", n->typed_id);
		j_key_u64(&j, "element_id", n->element_id);
		j_key_ustr(&j, "source_name", n->name);
		j_key_str(&j, "element_type", elem_type_name(n->element.type));
		j_key_str(&j, "attrib_type", elem_type_name(n->attrib_type));
		j_key_i64(&j, "parent_index", node_index(scene, n->parent));
		if (n->parent)
			j_key_ustr(&j, "parent_name", n->parent->name);
		else
			j_key_null(&j, "parent_name");
		j_key_bool(&j, "is_root", n->is_root);
		j_key_u64(&j, "node_depth", n->node_depth);
		j_key_bool(&j, "has_bone_attribute", n->bone != NULL);
		j_key_bool(&j, "has_mesh", n->mesh != NULL);
		j_key_bool(&j, "is_skin_cluster_bone", is_cluster[i]);
		j_key_bool(&j, "is_geometry_transform_helper", n->is_geometry_transform_helper);
		j_key_bool(&j, "is_scale_helper", n->is_scale_helper);
		j_key_bool(&j, "has_geometry_transform", n->has_geometry_transform);
		j_key_bool(&j, "use_rotation_space", n->use_rotation_space);
		j_key_bool(&j, "visible", n->visible);
		if (n->bone) {
			j_key_obj(&j, "bone");
			j_key_f(&j, "radius", n->bone->radius);
			j_key_f(&j, "relative_length", n->bone->relative_length);
			j_key_bool(&j, "is_root_bone", n->bone->is_root);
			j_end_obj(&j);
		} else {
			j_key_null(&j, "bone");
		}
		j_key_str(&j, "inherit_mode", inherit_name(n->inherit_mode));
		j_key_str(&j, "original_inherit_mode", inherit_name(n->original_inherit_mode));
		j_key_str(&j, "rotation_order", rot_order_name(n->rotation_order));
		j_key_vec3(&j, "euler_rotation_deg", n->euler_rotation);
		emit_authored_props(&j, n);
		j_key_obj(&j, "ufbx_evaluated_default");
		j_key_str(&j, "layer", "UFBX EVALUATED RESULT at load (default/rest-like pose, NOT a fabricated bind pose)");
		j_key_str(&j, "local_transform_api", "ufbx_node.local_transform");
		j_key_str(&j, "local_transform_note", "Evaluated local TRS. Not the authored pivot/pre/post recipe. Do not treat as plain Lcl TRS.");
		j_key_transform(&j, "local_transform", n->local_transform);
		j_key_str(&j, "node_to_parent_api", "ufbx_node.node_to_parent");
		j_key_matrix(&j, "node_to_parent", n->node_to_parent);
		j_key_str(&j, "node_to_world_api", "ufbx_node.node_to_world");
		j_key_str(&j, "node_to_world_note", "ufbx documents this as the product of parent-chain node_to_parent matrices. This is NOT claimed to equal ParentWorld * Lcl TRS.");
		j_key_matrix(&j, "node_to_world", n->node_to_world);
		j_key_transform(&j, "geometry_transform", n->geometry_transform);
		j_key_matrix(&j, "geometry_to_world", n->geometry_to_world);
		if (n->bind_pose)
			j_key_ustr(&j, "node_bind_pose_name", n->bind_pose->name);
		else
			j_key_null(&j, "node_bind_pose_name");
		j_end_obj(&j);
		j_end_obj(&j);
	}
	j_end_arr(&j);

	j_key_arr(&j, "meshes");
	for (size_t i = 0; i < scene->meshes.count; i++) {
		ufbx_mesh *m = scene->meshes.data[i];
		j_sep(&j);
		j_begin_obj(&j);
		j_key_ustr(&j, "name", m->name);
		j_key_sz(&j, "num_vertices", m->num_vertices);
		j_key_sz(&j, "num_faces", m->num_faces);
		j_key_sz(&j, "skin_deformer_count", m->skin_deformers.count);
		j_key_sz(&j, "instance_count", m->instances.count);
		j_key_arr(&j, "instance_nodes");
		for (size_t k = 0; k < m->instances.count; k++) {
			ufbx_node *inst = m->instances.data[k];
			j_sep(&j);
			j_begin_obj(&j);
			j_key_i64(&j, "index", node_index(scene, inst));
			j_key_ustr(&j, "name", inst->name);
			j_end_obj(&j);
		}
		j_end_arr(&j);
		j_end_obj(&j);
	}
	j_end_arr(&j);

	j_key_arr(&j, "skins");
	for (size_t i = 0; i < scene->skin_deformers.count; i++) {
		ufbx_skin_deformer *skin = scene->skin_deformers.data[i];
		j_sep(&j);
		j_begin_obj(&j);
		j_key_ustr(&j, "name", skin->name);
		j_key_sz(&j, "cluster_count", skin->clusters.count);
		j_key_sz(&j, "max_weights_per_vertex", skin->max_weights_per_vertex);
		j_key_arr(&j, "clusters");
		for (size_t c = 0; c < skin->clusters.count; c++) {
			ufbx_skin_cluster *cl = skin->clusters.data[c];
			j_sep(&j);
			j_begin_obj(&j);
			j_key_ustr(&j, "cluster_name", cl->name);
			if (cl->bone_node) {
				j_key_i64(&j, "bone_node_index", node_index(scene, cl->bone_node));
				j_key_ustr(&j, "bone_node_name", cl->bone_node->name);
			} else {
				j_key_null(&j, "bone_node_index");
				j_key_null(&j, "bone_node_name");
			}
			j_key_sz(&j, "num_weights", cl->num_weights);
			j_key_str(&j, "geometry_to_bone_api", "ufbx_skin_cluster.geometry_to_bone");
			j_key_matrix(&j, "geometry_to_bone", cl->geometry_to_bone);
			j_key_str(&j, "mesh_node_to_bone_api", "ufbx_skin_cluster.mesh_node_to_bone");
			j_key_matrix(&j, "mesh_node_to_bone", cl->mesh_node_to_bone);
			j_key_str(&j, "bind_to_world_api", "ufbx_skin_cluster.bind_to_world");
			j_key_str(&j, "bind_to_world_note", "ufbx: rest/bind pose transform of the node; prefer geometry_to_bone for skinning");
			j_key_matrix(&j, "bind_to_world", cl->bind_to_world);
			j_end_obj(&j);
		}
		j_end_arr(&j);
		emit_skin_vertex_weights(&j, skin);
		j_end_obj(&j);
	}
	j_end_arr(&j);

	j_key_arr(&j, "poses");
	for (size_t i = 0; i < scene->poses.count; i++) {
		ufbx_pose *p = scene->poses.data[i];
		j_sep(&j);
		j_begin_obj(&j);
		j_key_ustr(&j, "name", p->name);
		j_key_bool(&j, "is_bind_pose", p->is_bind_pose);
		j_key_sz(&j, "bone_pose_count", p->bone_poses.count);
		j_key_str(&j, "bone_to_parent_note", "ufbx: FBX stores world pose; bone_to_parent is approximated from parent world");
		j_end_obj(&j);
	}
	j_end_arr(&j);
	{
		int any_bind = 0;
		for (size_t i = 0; i < scene->poses.count; i++) {
			if (scene->poses.data[i]->is_bind_pose)
				any_bind = 1;
		}
		j_key_bool(&j, "explicit_fbx_bind_pose_present", any_bind);
		if (!any_bind)
			j_key_str(&j, "explicit_fbx_bind_pose_note", "No ufbx_pose with is_bind_pose. Not fabricated. Skin cluster bind matrices may still exist.");
	}

	j_key_obj(&j, "bind_rest_discipline");
	j_key_str(&j, "source_default_evaluated_pose", "ufbx_node.local_transform / node_to_world at load");
	j_key_str(&j, "fbx_pose_bind_information", "scene.poses[] with is_bind_pose");
	j_key_str(&j, "skin_cluster_bind_information", "geometry_to_bone / bind_to_world");
	j_key_str(&j, "rigforge_future_rest_pose", "NOT COMPUTED");
	j_key_str(&j, "do_not_label_all_as", "rest pose");
	j_end_obj(&j);

	j_key_arr(&j, "characters");
	for (size_t i = 0; i < scene->characters.count; i++) {
		j_sep(&j);
		j_begin_obj(&j);
		j_key_ustr(&j, "name", scene->characters.data[i]->name);
		j_end_obj(&j);
	}
	j_end_arr(&j);
	j_key_arr(&j, "constraints");
	for (size_t i = 0; i < scene->constraints.count; i++) {
		j_sep(&j);
		j_begin_obj(&j);
		j_key_ustr(&j, "name", scene->constraints.data[i]->name);
		j_end_obj(&j);
	}
	j_end_arr(&j);

	j_key_obj(&j, "animation");
	j_key_str(&j, "layer_note", "RAW SOURCE STRUCTURE. Stacks/layers are not flattened before capture.");
	j_key_arr(&j, "stacks");
	for (size_t i = 0; i < scene->anim_stacks.count; i++) {
		ufbx_anim_stack *st = scene->anim_stacks.data[i];
		j_sep(&j);
		j_begin_obj(&j);
		j_key_sz(&j, "index", i);
		j_key_ustr(&j, "name", st->name);
		j_key_f(&j, "time_begin", st->time_begin);
		j_key_f(&j, "time_end", st->time_end);
		j_key_sz(&j, "layer_count", st->layers.count);
		j_key_arr(&j, "layers");
		for (size_t li = 0; li < st->layers.count; li++) {
			ufbx_anim_layer *ly = st->layers.data[li];
			j_sep(&j);
			j_begin_obj(&j);
			j_key_ustr(&j, "name", ly->name);
			j_key_f(&j, "weight", ly->weight);
			j_key_bool(&j, "weight_is_animated", ly->weight_is_animated);
			j_key_bool(&j, "blended", ly->blended);
			j_key_bool(&j, "additive", ly->additive);
			j_key_bool(&j, "compose_rotation", ly->compose_rotation);
			j_key_bool(&j, "compose_scale", ly->compose_scale);
			j_key_sz(&j, "anim_props_count", ly->anim_props.count);
			j_key_sz(&j, "anim_values_count", ly->anim_values.count);
			j_key_arr(&j, "anim_props");
			size_t dump_n = ly->anim_props.count;
			int truncated = 0;
			if (dump_n > MAX_ANIM_PROP_DUMP) {
				dump_n = MAX_ANIM_PROP_DUMP;
				truncated = 1;
			}
			for (size_t pi = 0; pi < dump_n; pi++) {
				ufbx_anim_prop *ap = &ly->anim_props.data[pi];
				j_sep(&j);
				j_begin_obj(&j);
				if (ap->element)
					j_key_ustr(&j, "element_name", ap->element->name);
				else
					j_key_null(&j, "element_name");
				if (ap->element)
					j_key_str(&j, "element_type", elem_type_name(ap->element->type));
				j_key_ustr(&j, "prop_name", ap->prop_name);
				j_end_obj(&j);
			}
			j_end_arr(&j);
			j_key_bool(&j, "anim_props_truncated", truncated);
			if (truncated)
				j_key_str(&j, "truncation_class", "HARNESS GAP (size limit), not UFBX GAP");
			j_end_obj(&j);
		}
		j_end_arr(&j);
		j_end_obj(&j);
	}
	j_end_arr(&j);
	j_end_obj(&j);

	/* motion samples */
	ufbx_anim_stack *sample_stack = NULL;
	if (scene->anim_stacks.count > 0) {
		if (stack_index < 0 || (size_t)stack_index >= scene->anim_stacks.count)
			stack_index = 0;
		sample_stack = scene->anim_stacks.data[stack_index];
	}

	j_key_obj(&j, "evaluated_motion_samples");
	j_key_str(&j, "layer", "UFBX EVALUATED RESULT via ufbx_evaluate_scene / ufbx_evaluate_transform");
	if (!sample_stack) {
		j_key_null(&j, "stack");
		j_key_str(&j, "note", "No animation stack. Default-pose local/world is under nodes[].ufbx_evaluated_default.");
	} else {
		j_key_ustr(&j, "stack_name", sample_stack->name);
		j_key_i64(&j, "stack_index", stack_index);
		j_key_str(&j, "stack_selection", "scene->anim_stacks array order; not an assumption of a single stack");
		double t0 = sample_stack->time_begin;
		double t1 = sample_stack->time_end;
		double span = t1 - t0;
		double times[5];
		const char *labels[5] = { "start", "25pct", "50pct", "75pct", "end" };
		times[0] = t0;
		times[1] = t0 + span * 0.25;
		times[2] = t0 + span * 0.50;
		times[3] = t0 + span * 0.75;
		times[4] = t1;
		j_key_arr(&j, "times_seconds");
		for (int ti = 0; ti < 5; ti++) {
			j_sep(&j);
			j_begin_obj(&j);
			j_key_str(&j, "label", labels[ti]);
			j_key_f(&j, "t", times[ti]);
			j_end_obj(&j);
		}
		j_end_arr(&j);

		j_key_arr(&j, "nodes");
		for (int s = 0; s < auto_n; s++) {
			int ni = auto_idx[s];
			ufbx_node *n = scene->nodes.data[ni];
			if (sn.count > 0 && !want_sample(&sn, n->name, ni, auto_idx, auto_n))
				continue;
			if (sn.count == 0 && !want_sample(&sn, n->name, ni, auto_idx, auto_n))
				continue;
			j_sep(&j);
			j_begin_obj(&j);
			j_key_i64(&j, "traversal_index", ni);
			j_key_ustr(&j, "source_name", n->name);
			j_key_str(&j, "role_label", auto_role(s));
			j_key_str(&j, "role_justification", "Structural sample subset (depth / cluster-child count). Not a Canonical anatomical role. Not labelled pelvis.");
			j_key_arr(&j, "samples");
			for (int ti = 0; ti < 5; ti++) {
				ufbx_error ev_err;
				memset(&ev_err, 0, sizeof ev_err);
				ufbx_scene *ev = ufbx_evaluate_scene(scene, sample_stack->anim, times[ti], NULL, &ev_err);
				j_sep(&j);
				j_begin_obj(&j);
				j_key_str(&j, "label", labels[ti]);
				j_key_f(&j, "t", times[ti]);
				if (!ev) {
					j_key_bool(&j, "evaluate_scene_ok", 0);
					j_key_ustr(&j, "error", ev_err.description);
				} else {
					ufbx_node *en = ev->nodes.data[ni];
					j_key_bool(&j, "evaluate_scene_ok", 1);
					j_key_str(&j, "local_api", "ufbx_evaluate_scene -> node->local_transform");
					j_key_transform(&j, "local_transform", en->local_transform);
					j_key_str(&j, "world_api", "ufbx_evaluate_scene -> node->node_to_world");
					j_key_matrix(&j, "node_to_world", en->node_to_world);
					ufbx_transform lt = ufbx_evaluate_transform(sample_stack->anim, n, times[ti]);
					j_key_str(&j, "evaluate_transform_api", "ufbx_evaluate_transform(anim, original_node, time)");
					j_key_transform(&j, "evaluate_transform_local", lt);
					ufbx_free_scene(ev);
				}
				j_end_obj(&j);
			}
			j_end_arr(&j);
			j_end_obj(&j);
		}
		/* if user supplied names, dump those even if not in auto set */
		if (sn.count > 0) {
			for (size_t i = 0; i < scene->nodes.count; i++) {
				ufbx_node *n = scene->nodes.data[i];
				int in_auto = 0;
				for (int s = 0; s < auto_n; s++) {
					if (auto_idx[s] == (int)i)
						in_auto = 1;
				}
				if (in_auto)
					continue;
				int match = 0;
				for (int k = 0; k < sn.count; k++) {
					if (ustr_eq(n->name, sn.names[k]))
						match = 1;
				}
				if (!match)
					continue;
				j_sep(&j);
				j_begin_obj(&j);
				j_key_sz(&j, "traversal_index", i);
				j_key_ustr(&j, "source_name", n->name);
				j_key_str(&j, "role_label", "cli_sample_name");
				j_key_arr(&j, "samples");
				double t0b = sample_stack->time_begin;
				double t1b = sample_stack->time_end;
				double spanb = t1b - t0b;
				double timesb[5] = { t0b, t0b + spanb * 0.25, t0b + spanb * 0.5, t0b + spanb * 0.75, t1b };
				const char *labelsb[5] = { "start", "25pct", "50pct", "75pct", "end" };
				for (int ti = 0; ti < 5; ti++) {
					ufbx_error ev_err;
					memset(&ev_err, 0, sizeof ev_err);
					ufbx_scene *ev = ufbx_evaluate_scene(scene, sample_stack->anim, timesb[ti], NULL, &ev_err);
					j_sep(&j);
					j_begin_obj(&j);
					j_key_str(&j, "label", labelsb[ti]);
					j_key_f(&j, "t", timesb[ti]);
					if (!ev) {
						j_key_bool(&j, "evaluate_scene_ok", 0);
					} else {
						ufbx_node *en = ev->nodes.data[i];
						j_key_bool(&j, "evaluate_scene_ok", 1);
						j_key_transform(&j, "local_transform", en->local_transform);
						j_key_matrix(&j, "node_to_world", en->node_to_world);
						ufbx_free_scene(ev);
					}
					j_end_obj(&j);
				}
				j_end_arr(&j);
				j_end_obj(&j);
			}
		}
		j_end_arr(&j);
	}
	j_end_obj(&j);

	j_key_obj(&j, "derived_test_view");
	j_key_str(&j, "status", "NOT_CREATED");
	j_key_str(&j, "note", "Harness does not emit a normalized comparison representation.");
	j_end_obj(&j);

	j_key_obj(&j, "generality_guards");
	j_key_bool(&j, "assumes_humanoid", 0);
	j_key_bool(&j, "assumes_single_root", 0);
	j_key_bool(&j, "assumes_one_anim_stack", 0);
	j_key_bool(&j, "assumes_one_layer", 0);
	j_key_bool(&j, "assumes_root_is_pelvis", 0);
	j_key_bool(&j, "assumes_centimeters", 0);
	j_key_bool(&j, "assumes_y_up", 0);
	j_key_bool(&j, "assumes_one_mesh", 0);
	j_key_bool(&j, "assumes_one_skin", 0);
	j_key_bool(&j, "assumes_every_bone_deforms", 0);
	j_end_obj(&j);

	j_end_obj(&j);
	fputc('\n', out);
	fclose(out);
	free(is_cluster);
	ufbx_free_scene(scene);
	return 0;
}
