/*
 * RigForge POC-CORE-01 C++ worker
 *
 * RESEARCH ONLY / W0-P / NON-PRODUCTION
 * stdin/stdout UTF-8 JSON. See ../worker/protocol.md
 */

#include "rigforge_poc_core.h"

#include <cstdint>
#include <cstdio>
#include <cstring>
#include <iostream>
#include <string>
#include <vector>

namespace {

const char kB64[] =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

std::string b64_encode(const std::string &in)
{
    std::string out;
    size_t i = 0;
    unsigned char a3[3];
    while (i < in.size()) {
        size_t n = 0;
        a3[0] = a3[1] = a3[2] = 0;
        while (n < 3 && i < in.size()) {
            a3[n++] = static_cast<unsigned char>(in[i++]);
        }
        unsigned int v = (static_cast<unsigned int>(a3[0]) << 16) |
                         (static_cast<unsigned int>(a3[1]) << 8) |
                         static_cast<unsigned int>(a3[2]);
        out.push_back(kB64[(v >> 18) & 63]);
        out.push_back(kB64[(v >> 12) & 63]);
        out.push_back(n > 1 ? kB64[(v >> 6) & 63] : '=');
        out.push_back(n > 2 ? kB64[v & 63] : '=');
    }
    return out;
}

int b64_val(char c)
{
    if (c >= 'A' && c <= 'Z') {
        return c - 'A';
    }
    if (c >= 'a' && c <= 'z') {
        return c - 'a' + 26;
    }
    if (c >= '0' && c <= '9') {
        return c - '0' + 52;
    }
    if (c == '+') {
        return 62;
    }
    if (c == '/') {
        return 63;
    }
    return -1;
}

bool b64_decode(const std::string &in, std::string *out)
{
    out->clear();
    int val = 0;
    int bits = -8;
    for (char c : in) {
        if (c == '=' || c == '\n' || c == '\r') {
            continue;
        }
        int d = b64_val(c);
        if (d < 0) {
            return false;
        }
        val = (val << 6) + d;
        bits += 6;
        if (bits >= 0) {
            out->push_back(static_cast<char>((val >> bits) & 0xFF));
            bits -= 8;
        }
    }
    return true;
}

std::string json_escape(const std::string &s)
{
    std::string o;
    for (unsigned char c : s) {
        if (c == '\\' || c == '"') {
            o.push_back('\\');
            o.push_back(static_cast<char>(c));
        } else if (c == '\n') {
            o += "\\n";
        } else {
            o.push_back(static_cast<char>(c));
        }
    }
    return o;
}

bool extract_string_field(const std::string &json, const char *key, std::string *value)
{
    std::string pat = std::string("\"") + key + "\":\"";
    size_t p = json.find(pat);
    if (p == std::string::npos) {
        return false;
    }
    p += pat.size();
    value->clear();
    while (p < json.size()) {
        char c = json[p++];
        if (c == '\\' && p < json.size()) {
            value->push_back(json[p++]);
            continue;
        }
        if (c == '"') {
            return true;
        }
        value->push_back(c);
    }
    return false;
}

std::string fail_json(const char *code, const char *msg)
{
    return std::string("{\"ok\":false,\"payload_b64\":\"\",\"diag\":[{\"severity\":\"error\",\"code\":\"") +
           code + "\",\"message\":\"" + json_escape(msg) + "\",\"location\":\"\"}]}";
}

std::string diag_json(RF_PocAsset *a)
{
    std::string out = "[";
    int32_t n = rf_poc_diagnostic_count(a);
    for (int32_t i = 0; i < n; ++i) {
        int32_t sev = 0;
        size_t cl = 0, ml = 0, ll = 0;
        rf_poc_diagnostic(a, i, &sev, nullptr, 0, &cl, nullptr, 0, &ml, nullptr, 0, &ll);
        std::string code(cl, '\0'), msg(ml, '\0'), loc(ll, '\0');
        rf_poc_diagnostic(
            a, i, &sev,
            code.empty() ? nullptr : &code[0], cl + 1, &cl,
            msg.empty() ? nullptr : &msg[0], ml + 1, &ml,
            loc.empty() ? nullptr : &loc[0], ll + 1, &ll);
        code.resize(cl);
        msg.resize(ml);
        loc.resize(ll);
        const char *sev_s = "info";
        if (sev == RF_POC_SEV_WARN) {
            sev_s = "warn";
        } else if (sev == RF_POC_SEV_ERROR) {
            sev_s = "error";
        }
        if (i) {
            out += ",";
        }
        out += "{\"severity\":\"";
        out += sev_s;
        out += "\",\"code\":\"";
        out += json_escape(code);
        out += "\",\"message\":\"";
        out += json_escape(msg);
        out += "\",\"location\":\"";
        out += json_escape(loc);
        out += "\"}";
    }
    out += "]";
    return out;
}

} /* namespace */

int main()
{
    std::string line;
    if (!std::getline(std::cin, line)) {
        std::cout << fail_json("POC_BAD_OP", "empty stdin") << std::endl;
        return 0;
    }
    std::string op;
    if (!extract_string_field(line, "op", &op) || op != "load") {
        std::cout << fail_json("POC_BAD_OP", "expected op load") << std::endl;
        return 0;
    }
    std::string b64;
    if (!extract_string_field(line, "payload_b64", &b64)) {
        std::cout << fail_json("POC_BAD_B64", "missing payload_b64") << std::endl;
        return 0;
    }
    std::string path;
    if (!b64_decode(b64, &path) || path.empty()) {
        std::cout << fail_json("POC_BAD_B64", "invalid payload_b64") << std::endl;
        return 0;
    }

    rf_poc_status st = RF_POC_OK;
    RF_PocAsset *a = rf_poc_load(path.c_str(), &st);
    if (!a) {
        const char *code = "POC_PARSE";
        const char *msg = "load failed";
        if (st == RF_POC_ERR_NULL) {
            code = "POC_NULL";
            msg = "null path";
        } else if (st == RF_POC_ERR_IO) {
            code = "POC_IO";
            msg = "file not found or unreadable";
        }
        std::cout << fail_json(code, msg) << std::endl;
        return 0;
    }

    int32_t jc = rf_poc_joint_count(a);
    int32_t mc = rf_poc_motion_count(a);
    int32_t dc = rf_poc_diagnostic_count(a);
    double t0 = 0, t1 = 0;
    rf_poc_motion_time_range(a, 0, &t0, &t1);

    char inner[256];
    std::snprintf(
        inner, sizeof(inner),
        "{\"joint_count\":%d,\"motion_count\":%d,\"diag_count\":%d,\"t0\":%.1f,\"t1\":%.1f}",
        (int)jc, (int)mc, (int)dc, t0, t1);
    std::string payload = b64_encode(inner);
    std::string diags = diag_json(a);
    rf_poc_destroy(a);

    std::cout << "{\"ok\":true,\"payload_b64\":\"" << payload << "\",\"diag\":" << diags << "}"
              << std::endl;
    return 0;
}
