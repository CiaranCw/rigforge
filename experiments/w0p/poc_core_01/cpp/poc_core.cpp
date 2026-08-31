/*
 * RigForge POC-CORE-01 C++ research implementation
 *
 * RESEARCH ONLY / W0-P / NON-PRODUCTION
 * POC ONLY — NOT CANONICAL
 *
 * See ../ALGORITHM.md for the shared extraction procedure.
 */

#include "rigforge_poc_core.h"

#include "ufbx.h"

#include <cstring>
#include <mutex>
#include <new>
#include <string>
#include <unordered_set>
#include <vector>

namespace {

constexpr const char *kAssetId = "poc_core_01_asset";

struct Vec3 {
    double x = 0, y = 0, z = 0;
};

struct Quat {
    double x = 0, y = 0, z = 0, w = 1;
};

struct TrsKey {
    double time = 0;
    Vec3 t;
};

struct RotKey {
    double time = 0;
    Quat q;
};

struct Joint {
    std::string name;
    int32_t parent = -1;
    Vec3 t;
    Quat r;
    Vec3 s{1, 1, 1};
};

struct Track {
    int32_t joint = 0;
    std::vector<TrsKey> t_keys;
    std::vector<RotKey> r_keys;
};

struct Motion {
    std::string name;
    double t0 = 0;
    double t1 = 1;
    std::vector<Track> tracks;
};

struct Diagnostic {
    int32_t severity = 0;
    std::string code;
    std::string message;
    std::string location;
};

struct Asset {
    uint32_t magic = 0x52465043u; /* 'RFPC' */
    std::string id = kAssetId;
    std::vector<Joint> joints;
    std::vector<Motion> motions;
    std::vector<Diagnostic> diags;
};

std::mutex g_mu;
/* Research harness only: pointer-address live registry.
 * Validates immediate post-destroy misuse on the single-threaded tested path.
 * NOT a production stale-handle / ABA / concurrent query-destroy design. */
std::unordered_set<Asset *> g_live;

bool live(Asset *a)
{
    if (!a) {
        return false;
    }
    std::lock_guard<std::mutex> lock(g_mu);
    return g_live.find(a) != g_live.end();
}

Asset *as_asset(RF_PocAsset *h)
{
    return reinterpret_cast<Asset *>(h);
}

rf_poc_status copy_str(const std::string &s, char *buf, size_t buf_len, size_t *out_len)
{
    if (out_len) {
        *out_len = s.size();
    }
    if (!buf) {
        return RF_POC_OK;
    }
    if (buf_len < s.size() + 1) {
        return RF_POC_ERR_BUFFER;
    }
    if (!s.empty()) {
        std::memcpy(buf, s.data(), s.size());
    }
    buf[s.size()] = '\0';
    return RF_POC_OK;
}

rf_poc_status copy_str3(
    const std::string &code, const std::string &msg, const std::string &loc,
    char *code_buf, size_t code_len, size_t *code_out,
    char *msg_buf, size_t msg_len, size_t *msg_out,
    char *loc_buf, size_t loc_len, size_t *loc_out)
{
    rf_poc_status st = copy_str(code, code_buf, code_len, code_out);
    if (st != RF_POC_OK) {
        return st;
    }
    st = copy_str(msg, msg_buf, msg_len, msg_out);
    if (st != RF_POC_OK) {
        return st;
    }
    return copy_str(loc, loc_buf, loc_len, loc_out);
}

void attach_synth(Asset *a)
{
    Motion m;
    m.name = "poc_walk";
    m.t0 = 0.0;
    m.t1 = 1.0;
    const int32_t n = static_cast<int32_t>(a->joints.size());
    for (int32_t i = 0; i < n; ++i) {
        Track tr;
        tr.joint = i;
        const Joint &j = a->joints[static_cast<size_t>(i)];
        TrsKey k0;
        k0.time = 0.0;
        k0.t = j.t;
        TrsKey k1;
        k1.time = 1.0;
        k1.t = j.t;
        k1.t.x += 0.1 * static_cast<double>(i);
        tr.t_keys.push_back(k0);
        tr.t_keys.push_back(k1);
        if (i != n - 1) {
            RotKey r0;
            r0.time = 0.0;
            r0.q = j.r;
            RotKey r1;
            r1.time = 1.0;
            r1.q = j.r;
            tr.r_keys.push_back(r0);
            tr.r_keys.push_back(r1);
        }
        m.tracks.push_back(std::move(tr));
    }
    a->motions.push_back(std::move(m));
}

void attach_diags(Asset *a, const std::string &path)
{
    Diagnostic d0;
    d0.severity = RF_POC_SEV_INFO;
    d0.code = "POC_LOAD";
    d0.message = "ufbx load ok";
    d0.location = path;
    a->diags.push_back(std::move(d0));

    Diagnostic d1;
    d1.severity = RF_POC_SEV_INFO;
    d1.code = "POC_JOINTS";
    d1.message = std::to_string(a->joints.size()) + " joints extracted";
    a->diags.push_back(std::move(d1));

    Diagnostic d2;
    d2.severity = RF_POC_SEV_WARN;
    d2.code = "POC_SYNTH";
    d2.message = "native FBX animation unused; synthetic research motion attached";
    a->diags.push_back(std::move(d2));

    Diagnostic d3;
    d3.severity = RF_POC_SEV_INFO;
    d3.code = "POC_EMPTY_ROT";
    d3.message = "last joint rotation keys empty by design";
    if (!a->joints.empty()) {
        d3.location = a->joints.back().name;
    }
    a->diags.push_back(std::move(d3));
}

RF_PocAsset *load_impl(const char *path, rf_poc_status *out_status)
{
    if (out_status) {
        *out_status = RF_POC_OK;
    }
    if (!path) {
        if (out_status) {
            *out_status = RF_POC_ERR_NULL;
        }
        return nullptr;
    }

    ufbx_load_opts opts;
    std::memset(&opts, 0, sizeof(opts));
    ufbx_error error;
    std::memset(&error, 0, sizeof(error));
    ufbx_scene *scene = ufbx_load_file(path, &opts, &error);
    if (!scene) {
        if (out_status) {
            *out_status = (error.type == UFBX_ERROR_FILE_NOT_FOUND)
                ? RF_POC_ERR_IO
                : RF_POC_ERR_PARSE;
        }
        return nullptr;
    }

    Asset *a = new (std::nothrow) Asset();
    if (!a) {
        ufbx_free_scene(scene);
        if (out_status) {
            *out_status = RF_POC_ERR_PARSE;
        }
        return nullptr;
    }

    std::vector<ufbx_node *> nodes;
    nodes.reserve(scene->nodes.count);
    for (size_t i = 0; i < scene->nodes.count; ++i) {
        ufbx_node *n = scene->nodes.data[i];
        if (!n || n->is_root) {
            continue;
        }
        nodes.push_back(n);
    }

    a->joints.reserve(nodes.size());
    for (ufbx_node *n : nodes) {
        Joint j;
        if (n->name.data && n->name.length > 0) {
            j.name.assign(n->name.data, n->name.length);
        } else {
            j.name = "unnamed";
        }
        j.t.x = n->local_transform.translation.x;
        j.t.y = n->local_transform.translation.y;
        j.t.z = n->local_transform.translation.z;
        j.r.x = n->local_transform.rotation.x;
        j.r.y = n->local_transform.rotation.y;
        j.r.z = n->local_transform.rotation.z;
        j.r.w = n->local_transform.rotation.w;
        j.s.x = n->local_transform.scale.x;
        j.s.y = n->local_transform.scale.y;
        j.s.z = n->local_transform.scale.z;
        a->joints.push_back(std::move(j));
    }

    for (size_t i = 0; i < nodes.size(); ++i) {
        ufbx_node *p = nodes[i]->parent;
        int32_t parent = -1;
        if (p && !p->is_root) {
            for (size_t k = 0; k < nodes.size(); ++k) {
                if (nodes[k] == p) {
                    parent = static_cast<int32_t>(k);
                    break;
                }
            }
        }
        a->joints[i].parent = parent;
    }

    ufbx_free_scene(scene);
    scene = nullptr;

    attach_synth(a);
    attach_diags(a, path);

    {
        std::lock_guard<std::mutex> lock(g_mu);
        g_live.insert(a);
    }
    return reinterpret_cast<RF_PocAsset *>(a);
}

/* PoC-only C ABI unwind boundary. Not a production exception policy.
 * Unexpected internal exceptions map to RF_POC_ERR_PARSE / -1 / swallowed destroy. */
template <typename Fn>
rf_poc_status abi_status(Fn &&fn)
{
    try {
        return fn();
    } catch (...) {
        return RF_POC_ERR_PARSE;
    }
}

template <typename Fn>
int32_t abi_count(Fn &&fn)
{
    try {
        return fn();
    } catch (...) {
        return -1;
    }
}

template <typename Fn>
void abi_void(Fn &&fn)
{
    try {
        fn();
    } catch (...) {
    }
}

} /* namespace */

extern "C" {

RF_PocAsset *rf_poc_load(const char *path, rf_poc_status *out_status)
{
    try {
        return load_impl(path, out_status);
    } catch (...) {
        if (out_status) {
            *out_status = RF_POC_ERR_PARSE;
        }
        return nullptr;
    }
}

void rf_poc_destroy(RF_PocAsset *asset)
{
    abi_void([&] {
        Asset *a = as_asset(asset);
        if (!a) {
            return;
        }
        {
            std::lock_guard<std::mutex> lock(g_mu);
            auto it = g_live.find(a);
            if (it == g_live.end()) {
                return;
            }
            g_live.erase(it);
        }
        a->magic = 0;
        delete a;
    });
}

rf_poc_status rf_poc_asset_id(RF_PocAsset *asset, char *buf, size_t buf_len, size_t *out_len)
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        return copy_str(a->id, buf, buf_len, out_len);
    });
}

int32_t rf_poc_joint_count(RF_PocAsset *asset)
{
    return abi_count([&]() -> int32_t {
        Asset *a = as_asset(asset);
        if (!asset || !live(a)) {
            return -1;
        }
        return static_cast<int32_t>(a->joints.size());
    });
}

rf_poc_status rf_poc_joint_parent(RF_PocAsset *asset, int32_t index, int32_t *out_parent)
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (!out_parent) {
            return RF_POC_ERR_NULL;
        }
        if (index < 0 || static_cast<size_t>(index) >= a->joints.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        *out_parent = a->joints[static_cast<size_t>(index)].parent;
        return RF_POC_OK;
    });
}

rf_poc_status rf_poc_joint_name(
    RF_PocAsset *asset, int32_t index,
    char *buf, size_t buf_len, size_t *out_len)
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (index < 0 || static_cast<size_t>(index) >= a->joints.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        return copy_str(a->joints[static_cast<size_t>(index)].name, buf, buf_len, out_len);
    });
}

rf_poc_status rf_poc_joint_rest_translation(RF_PocAsset *asset, int32_t index, double out_xyz[3])
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset || !out_xyz) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (index < 0 || static_cast<size_t>(index) >= a->joints.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Vec3 &t = a->joints[static_cast<size_t>(index)].t;
        out_xyz[0] = t.x;
        out_xyz[1] = t.y;
        out_xyz[2] = t.z;
        return RF_POC_OK;
    });
}

rf_poc_status rf_poc_joint_rest_rotation(RF_PocAsset *asset, int32_t index, double out_xyzw[4])
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset || !out_xyzw) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (index < 0 || static_cast<size_t>(index) >= a->joints.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Quat &r = a->joints[static_cast<size_t>(index)].r;
        out_xyzw[0] = r.x;
        out_xyzw[1] = r.y;
        out_xyzw[2] = r.z;
        out_xyzw[3] = r.w;
        return RF_POC_OK;
    });
}

rf_poc_status rf_poc_joint_rest_scale(RF_PocAsset *asset, int32_t index, double out_xyz[3])
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset || !out_xyz) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (index < 0 || static_cast<size_t>(index) >= a->joints.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Vec3 &s = a->joints[static_cast<size_t>(index)].s;
        out_xyz[0] = s.x;
        out_xyz[1] = s.y;
        out_xyz[2] = s.z;
        return RF_POC_OK;
    });
}

int32_t rf_poc_motion_count(RF_PocAsset *asset)
{
    return abi_count([&]() -> int32_t {
        Asset *a = as_asset(asset);
        if (!asset || !live(a)) {
            return -1;
        }
        return static_cast<int32_t>(a->motions.size());
    });
}

rf_poc_status rf_poc_motion_name(
    RF_PocAsset *asset, int32_t motion_index,
    char *buf, size_t buf_len, size_t *out_len)
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (motion_index < 0 || static_cast<size_t>(motion_index) >= a->motions.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        return copy_str(a->motions[static_cast<size_t>(motion_index)].name, buf, buf_len, out_len);
    });
}

rf_poc_status rf_poc_motion_time_range(
    RF_PocAsset *asset, int32_t motion_index,
    double *out_t0, double *out_t1)
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset || !out_t0 || !out_t1) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (motion_index < 0 || static_cast<size_t>(motion_index) >= a->motions.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        *out_t0 = a->motions[static_cast<size_t>(motion_index)].t0;
        *out_t1 = a->motions[static_cast<size_t>(motion_index)].t1;
        return RF_POC_OK;
    });
}

int32_t rf_poc_track_count(RF_PocAsset *asset, int32_t motion_index)
{
    return abi_count([&]() -> int32_t {
        Asset *a = as_asset(asset);
        if (!asset || !live(a)) {
            return -1;
        }
        if (motion_index < 0 || static_cast<size_t>(motion_index) >= a->motions.size()) {
            return -1;
        }
        return static_cast<int32_t>(a->motions[static_cast<size_t>(motion_index)].tracks.size());
    });
}

rf_poc_status rf_poc_track_joint(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index,
    int32_t *out_joint)
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset || !out_joint) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (motion_index < 0 || static_cast<size_t>(motion_index) >= a->motions.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Motion &m = a->motions[static_cast<size_t>(motion_index)];
        if (track_index < 0 || static_cast<size_t>(track_index) >= m.tracks.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        *out_joint = m.tracks[static_cast<size_t>(track_index)].joint;
        return RF_POC_OK;
    });
}

int32_t rf_poc_track_translation_key_count(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index)
{
    return abi_count([&]() -> int32_t {
        Asset *a = as_asset(asset);
        if (!asset || !live(a)) {
            return -1;
        }
        if (motion_index < 0 || static_cast<size_t>(motion_index) >= a->motions.size()) {
            return -1;
        }
        const Motion &m = a->motions[static_cast<size_t>(motion_index)];
        if (track_index < 0 || static_cast<size_t>(track_index) >= m.tracks.size()) {
            return -1;
        }
        return static_cast<int32_t>(m.tracks[static_cast<size_t>(track_index)].t_keys.size());
    });
}

int32_t rf_poc_track_rotation_key_count(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index)
{
    return abi_count([&]() -> int32_t {
        Asset *a = as_asset(asset);
        if (!asset || !live(a)) {
            return -1;
        }
        if (motion_index < 0 || static_cast<size_t>(motion_index) >= a->motions.size()) {
            return -1;
        }
        const Motion &m = a->motions[static_cast<size_t>(motion_index)];
        if (track_index < 0 || static_cast<size_t>(track_index) >= m.tracks.size()) {
            return -1;
        }
        return static_cast<int32_t>(m.tracks[static_cast<size_t>(track_index)].r_keys.size());
    });
}

rf_poc_status rf_poc_track_translation_key(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index, int32_t key_index,
    double *out_time, double out_xyz[3])
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset || !out_time || !out_xyz) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (motion_index < 0 || static_cast<size_t>(motion_index) >= a->motions.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Motion &m = a->motions[static_cast<size_t>(motion_index)];
        if (track_index < 0 || static_cast<size_t>(track_index) >= m.tracks.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Track &tr = m.tracks[static_cast<size_t>(track_index)];
        if (key_index < 0 || static_cast<size_t>(key_index) >= tr.t_keys.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const TrsKey &k = tr.t_keys[static_cast<size_t>(key_index)];
        *out_time = k.time;
        out_xyz[0] = k.t.x;
        out_xyz[1] = k.t.y;
        out_xyz[2] = k.t.z;
        return RF_POC_OK;
    });
}

rf_poc_status rf_poc_track_rotation_key(
    RF_PocAsset *asset, int32_t motion_index, int32_t track_index, int32_t key_index,
    double *out_time, double out_xyzw[4])
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset || !out_time || !out_xyzw) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (motion_index < 0 || static_cast<size_t>(motion_index) >= a->motions.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Motion &m = a->motions[static_cast<size_t>(motion_index)];
        if (track_index < 0 || static_cast<size_t>(track_index) >= m.tracks.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Track &tr = m.tracks[static_cast<size_t>(track_index)];
        if (key_index < 0 || static_cast<size_t>(key_index) >= tr.r_keys.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const RotKey &k = tr.r_keys[static_cast<size_t>(key_index)];
        *out_time = k.time;
        out_xyzw[0] = k.q.x;
        out_xyzw[1] = k.q.y;
        out_xyzw[2] = k.q.z;
        out_xyzw[3] = k.q.w;
        return RF_POC_OK;
    });
}

int32_t rf_poc_diagnostic_count(RF_PocAsset *asset)
{
    return abi_count([&]() -> int32_t {
        Asset *a = as_asset(asset);
        if (!asset || !live(a)) {
            return -1;
        }
        return static_cast<int32_t>(a->diags.size());
    });
}

rf_poc_status rf_poc_diagnostic(
    RF_PocAsset *asset, int32_t index,
    int32_t *out_severity,
    char *code_buf, size_t code_len, size_t *code_out,
    char *msg_buf, size_t msg_len, size_t *msg_out,
    char *loc_buf, size_t loc_len, size_t *loc_out)
{
    return abi_status([&]() -> rf_poc_status {
        Asset *a = as_asset(asset);
        if (!asset || !out_severity) {
            return RF_POC_ERR_NULL;
        }
        if (!live(a)) {
            return RF_POC_ERR_INVALID_HANDLE;
        }
        if (index < 0 || static_cast<size_t>(index) >= a->diags.size()) {
            return RF_POC_ERR_INVALID_INDEX;
        }
        const Diagnostic &d = a->diags[static_cast<size_t>(index)];
        *out_severity = d.severity;
        return copy_str3(
            d.code, d.message, d.location,
            code_buf, code_len, code_out,
            msg_buf, msg_len, msg_out,
            loc_buf, loc_len, loc_out);
    });
}

} /* extern "C" */
