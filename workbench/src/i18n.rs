//! User-facing Workbench language. Product identity, protocol, and logs stay English.

use rigforge_app::rigforge_domain::{
    CompatibilitySummary, Judgment, TimeDomainProvenance, TimeKind, UnmappedDisposition,
};
use rigforge_app::{
    TransferAuthorization, INGEST_FILE_CHANGED, INGEST_FRACTIONAL_FRAMES, INGEST_INSPECT_FAILED,
    INGEST_MISSING_FILE, INGEST_MULTIPLE_SKELETONS, INGEST_NO_CLIPS, INGEST_NO_SKELETON,
    INGEST_UNSUPPORTED_FORMAT, INGEST_UNUSABLE_TIMING,
};

pub const WINDOW_TITLE: &str = "RigForge 工作台";
pub const APP_NAME: &str = "RigForge 工作台";
pub const STATUS_LOCAL: &str = "本地运行";
pub const STATUS_NO_BLENDER_UI: &str = "无需打开 Blender";

pub const ASSET_BROWSER: &str = "资产库";
pub const CHARACTERS: &str = "角色";
pub const MOTIONS: &str = "动作";
pub const DERIVED_VARIANTS: &str = "派生资产";
pub const EMPTY_CHARACTERS: &str = "暂无角色资产";
pub const EMPTY_MOTIONS: &str = "暂无动作资产";
pub const EMPTY_DERIVED: &str = "暂无派生资产";

pub const DISPLAY_NAME: &str = "显示名称";
pub const BROWSE: &str = "浏览";
pub const CHANGE: &str = "更换";
pub const FILE: &str = "文件";
pub const NO_FBX: &str = "尚未选择 FBX 文件";
pub const SOURCE_SKELETON: &str = "来源骨架";
pub const ANIMATION_CLIP: &str = "动画片段";
pub const CHOOSE_CLIP: &str = "请选择动画片段";
pub const CLIP_HINT: &str = "选择动画片段后，再添加到资产库。选择片段本身不会注册动作。";
pub const ADD_CHARACTER_SECTION: &str = "添加角色";
pub const ADD_MOTION_SECTION: &str = "添加动作";
pub const BTN_ADD_CHARACTER: &str = "添加角色";
pub const BTN_ADD_MOTION: &str = "添加到资产库";

pub const STATUS_INSPECTING_CHARACTER: &str = "正在分析角色…";
pub const STATUS_INSPECTING_MOTION: &str = "正在分析动作…";
pub const STATUS_CHARACTER_READY: &str = "角色文件已分析完成，可以添加。";
pub const STATUS_MOTION_READY: &str = "动作文件已分析完成，可以添加。";
pub const STATUS_MULTI_CLIPS: &str = "发现多个动画片段，请选择一个后再添加到资产库。";

pub const CURRENT_SELECTION: &str = "当前选择";
pub const LABEL_CHARACTER: &str = "角色";
pub const LABEL_MOTION: &str = "动作";
pub const LABEL_CLIP: &str = "动画片段";
pub const LABEL_DERIVED: &str = "派生结果";
pub const NONE_SELECTED: &str = "尚未选择";
pub const DERIVED_NOT_CREATED: &str = "尚未生成";
pub const TECHNICAL_DETAILS: &str = "技术详情";

pub const WORKFLOW_GUIDE: &str =
    "工作流程：1. 生成映射  →  2. 查看并接受映射  →  3. 检查兼容性  →  4. 执行迁移  →  5. 预览结果";

pub const SECTION_MAPPING: &str = "1. 骨骼映射";
pub const SECTION_COMPAT: &str = "2. 兼容性检查";
pub const SECTION_TRANSFER: &str = "3. 执行迁移";
pub const SECTION_PREVIEW: &str = "4. 预览";
pub const SECTION_JOBS: &str = "任务状态";

pub const MAPPED_COUNT: &str = "已匹配";
pub const UNMAPPED_COUNT: &str = "未匹配";
pub const AMBIGUITY_COUNT: &str = "歧义";
pub const MAPPING_STATUS: &str = "映射状态";
pub const MAPPING_PENDING: &str = "待接受";
pub const MAPPING_ACCEPTED: &str = "已接受";
pub const MAPPING_NONE: &str = "尚未生成";
pub const MAPPING_DETAILS: &str = "映射详情";
pub const MAPPED_JOINTS: &str = "已匹配关节";
pub const UNMAPPED_JOINTS: &str = "未匹配关节";
pub const AMBIGUITIES: &str = "歧义";

pub const BTN_PROPOSE_MAPPING: &str = "生成映射";
pub const BTN_ACCEPT_MAPPING: &str = "接受映射";
pub const BTN_EVALUATE_COMPAT: &str = "检查兼容性";
pub const BTN_TRANSFER: &str = "执行迁移";

pub const COMPAT_OVERALL: &str = "总体结果";
pub const COMPAT_DETAILS: &str = "详细检查";
pub const COMPAT_WARNINGS: &str = "警告";
pub const TRANSFER_STATE: &str = "迁移状态";
pub const ELIGIBLE: &str = "可以执行";
pub const NEEDS_ACK: &str = "需要确认警告";
pub const NOT_ELIGIBLE: &str = "不可执行";
pub const ACK_WARNINGS: &str = "我已阅读并了解以上兼容性警告";

pub const ELAPSED_PREFIX: &str = "已用时";
pub const QC: &str = "质量检查";
pub const PERSISTENCE: &str = "持久化验证";
pub const PUBLICATION: &str = "发布状态";
pub const PUBLISHED: &str = "已发布";
pub const PUBLICATION_DENIED: &str = "未发布";
pub const NO_JOBS: &str = "暂无任务";

pub const PREVIEW: &str = "预览";
pub const BTN_PREVIEW_CHARACTER: &str = "预览角色";
pub const BTN_PREVIEW_MOTION: &str = "预览动作";
pub const BTN_PREVIEW_DERIVED: &str = "预览派生资产";
pub const BTN_REGENERATE_PREVIEW: &str = "重新生成预览";
pub const PREVIEW_SYNC_NOTE: &str = "当前预览生成为同步操作，生成期间窗口可能短暂等待。";
pub const PREVIEW_DERIVED_NOTE: &str = "预览为派生产物，可重建，不作为产品权威。";
pub const PREVIEW_PAYLOAD_NOTE: &str = "预览载荷，不是产品格式";
pub const PREVIEW_NONE: &str = "尚未生成预览";
pub const PREVIEW_VALID: &str = "有效";
pub const PREVIEW_INVALID: &str = "无效";
pub const PREVIEW_OCCUPIED: &str = "已占用";
pub const PREVIEW_EMPTY: &str = "空";
pub const VIEWER: &str = "查看器";
pub const PAYLOAD: &str = "载荷";
pub const STATUS: &str = "状态";

pub const ID_CHARACTER_VERSION: &str = "角色版本 ID";
pub const ID_MOTION_VERSION: &str = "动作版本 ID";
pub const ID_MAPPING_VERSION: &str = "MappingVersion ID";
pub const ID_COMPAT: &str = "CompatibilityResult ID";
pub const ID_DERIVED_VARIANT: &str = "DerivedVariant ID";
pub const ID_DERIVED_VERSION: &str = "DerivedVariantVersion ID";
pub const ID_JOB_RUN: &str = "JobRun ID";

pub const ERR_NO_CHARACTER: &str = "当前没有已选择的角色。";
pub const ERR_NO_MOTION: &str = "当前没有已选择的动作。";
pub const ERR_ACCEPT_MAPPING: &str = "请先接受骨骼映射。";
pub const ERR_NEED_COMPAT: &str = "请先完成兼容性检查。";
pub const ERR_NEED_APP_CHARACTER: &str = "添加角色需要已打开的本地工作台。";
pub const ERR_NEED_APP_MOTION: &str = "添加动作需要已打开的本地工作台。";
pub const ERR_NEED_APP_MAPPING: &str = "骨骼映射需要已打开的本地工作台。";
pub const ERR_NEED_APP_COMPAT: &str = "兼容性检查需要已打开的本地工作台。";
pub const ERR_NEED_APP_TRANSFER: &str = "执行迁移需要已打开的本地工作台。";
pub const ERR_NEED_APP_PREVIEW: &str = "预览需要已打开的本地工作台。";
pub const ERR_INSPECT_CHARACTER: &str = "请先分析角色 FBX，再添加到资产库。";
pub const ERR_INSPECT_MOTION: &str = "请先分析动作 FBX，再添加到资产库。";
pub const ERR_DISPLAY_NAME: &str = "请填写显示名称。";
pub const ERR_NO_MAPPING: &str = "当前没有可接受的骨骼映射。";

pub const MSG_MAPPING_DRAFT: &str = "已生成骨骼映射草案，请查看后接受。未接受前不能检查兼容性。";
pub const MSG_MAPPING_ACCEPTED: &str = "骨骼映射已接受。";
pub const PHASE_LAUNCHING: &str = "正在启动处理后端…";
pub const PHASE_RUNNING: &str = "正在执行动作迁移…";
pub const PHASE_VALIDATING: &str = "正在验证迁移结果…";
pub const PHASE_PERSISTENCE: &str = "正在验证已保存结果…";
pub const PHASE_PUBLISHING: &str = "正在发布派生资产…";
pub const PHASE_COMPLETE: &str = "迁移完成";
pub const PHASE_FAILED: &str = "迁移失败";

pub fn character_added_status(display_name: &str) -> String {
    format!("角色已添加并设为当前选择：{display_name}")
}

pub fn motion_added_status(display_name: &str, clip_label: &str) -> String {
    format!("动作已添加并设为当前选择：{display_name} · {clip_label}")
}

pub fn clip_friendly_label(identity: &str) -> String {
    let last = identity.rsplit('|').next().unwrap_or(identity);
    last.replace('_', " ").replace('-', " ")
}

pub fn format_time_range(time: &TimeDomainProvenance) -> Option<String> {
    let start = time.start();
    let end = time.end();
    if start.kind() != TimeKind::Frames || !start.is_integral_frame() || !end.is_integral_frame() {
        return None;
    }
    let frames = format!("{}–{} 帧", start.value_num(), end.value_num());
    let fps = match (start.fps_num(), start.fps_den()) {
        (Some(num), Some(1)) => format!("{num} FPS"),
        (Some(num), Some(den)) => format!("{num}/{den} FPS"),
        _ => return Some(frames),
    };
    Some(format!("{frames} · {fps}"))
}

pub fn clip_timing_zh(
    start: Option<i64>,
    end: Option<i64>,
    fps_num: Option<u32>,
    fps_den: Option<u32>,
) -> String {
    let frames = match (start, end) {
        (Some(a), Some(b)) => format!("{a}–{b} 帧"),
        _ => "帧范围不可用".to_string(),
    };
    let fps = match (fps_num, fps_den) {
        (Some(num), Some(1)) => format!("{num} FPS"),
        (Some(num), Some(den)) => format!("{num}/{den} FPS"),
        _ => "帧率不可用".to_string(),
    };
    format!("{frames} · {fps}")
}

pub fn unmapped_disposition_zh(disposition: UnmappedDisposition) -> &'static str {
    match disposition {
        UnmappedDisposition::Blocking => "阻断",
        UnmappedDisposition::Optional => "可选",
        UnmappedDisposition::Helper => "辅助",
    }
}

pub fn judgment_zh(value: &str) -> String {
    match value {
        "Pass" | "pass" => "通过".into(),
        "PassWithWarnings" => "通过（有警告）".into(),
        "Fail" | "fail" => "未通过".into(),
        "Unknown" | "unknown" => "未知".into(),
        other => other.to_string(),
    }
}

pub fn judgment_enum_zh(judgment: Judgment) -> &'static str {
    match judgment {
        Judgment::Pass => "通过",
        Judgment::PassWithWarnings => "通过（有警告）",
        Judgment::Fail => "未通过",
        Judgment::Unknown => "未知",
    }
}

pub fn summary_zh(value: &str) -> String {
    match value {
        "Ready" | "ready" => "可以迁移".into(),
        "ReadyWithWarnings" => "可迁移，但存在警告".into(),
        "MappingConfirmationRequired" => "需要确认骨骼映射".into(),
        "Unsupported" => "当前不支持迁移".into(),
        other => other.to_string(),
    }
}

pub fn summary_enum_zh(summary: CompatibilitySummary) -> &'static str {
    match summary {
        CompatibilitySummary::Ready => "可以迁移",
        CompatibilitySummary::ReadyWithWarnings => "可迁移，但存在警告",
        CompatibilitySummary::MappingConfirmationRequired => "需要确认骨骼映射",
        CompatibilitySummary::Unsupported => "当前不支持迁移",
    }
}

pub fn dimension_zh(name: &str) -> &str {
    match name {
        "mapping_completeness" => "映射完整度",
        "structural_compatibility" => "结构兼容性",
        "method_eligibility" => "方法适用性",
        "motion_suitability" => "动作适用性",
        "result_acceptability" => "结果可接受性",
        other => other,
    }
}

pub fn transfer_eligibility_zh(
    auth: Option<&TransferAuthorization>,
    warnings_acknowledged: bool,
) -> &'static str {
    match auth {
        Some(auth) if auth.eligible => ELIGIBLE,
        Some(auth) if auth.requires_acknowledgement && !warnings_acknowledged => NEEDS_ACK,
        Some(_) => NOT_ELIGIBLE,
        None => ERR_NEED_COMPAT,
    }
}

pub fn publication_zh(state: &str) -> &str {
    match state {
        "Published" => PUBLISHED,
        "Publication denied" => PUBLICATION_DENIED,
        other => other,
    }
}

pub fn job_state_zh(state: &str) -> &str {
    match state {
        "queued" | "QUEUED" => "排队中",
        "dispatchable" | "DISPATCHABLE" => "待调度",
        "running" | "RUNNING" => "运行中",
        "success" | "SUCCEEDED" | "SUCCESS" => "成功",
        "failed" | "FAILED" => "失败",
        "cancelled" | "CANCELLED" => "已取消",
        other => other,
    }
}

pub fn qc_zh(verdict: &str) -> &str {
    match verdict {
        "Pass" | "pass" => "通过",
        "Fail" | "fail" => "未通过",
        other => other,
    }
}

pub fn present_user_text(raw: &str) -> String {
    if let Some(mapped) = present_known(raw) {
        return mapped;
    }
    if let Some(rest) = raw.strip_prefix("Compatibility ") {
        return format!("兼容性检查：{}", summary_zh(rest));
    }
    if let Some(rest) = raw.strip_prefix("Preview generation failed: ") {
        return format!("预览生成失败：{}", present_user_text(rest));
    }
    if let Some(rest) = raw.strip_prefix("valid Preview for ") {
        return format!("已生成有效预览（{rest}）");
    }
    if raw.contains("background Transfer channel disconnected") {
        return "迁移通道意外中断。".into();
    }
    if raw.starts_with("sqlite:") || raw.starts_with("io:") || raw.starts_with("orchestration:") {
        return format!("内部错误：{raw}");
    }
    raw.to_string()
}

fn present_known(raw: &str) -> Option<String> {
    let mapped = match raw {
        INGEST_NO_SKELETON => "未在该 FBX 中发现可用骨架。",
        INGEST_MULTIPLE_SKELETONS => {
            "该 FBX 中发现多个可用骨架。当前版本要求每个来源文件只有一个明确骨架。"
        }
        INGEST_NO_CLIPS => "未找到可用动画片段。",
        INGEST_FRACTIONAL_FRAMES => "该动画使用了当前不支持的分数帧端点。",
        INGEST_FILE_CHANGED => "文件在分析后发生了变化，请重新分析后再添加。",
        INGEST_INSPECT_FAILED => "无法用当前处理后端分析该 FBX。",
        INGEST_UNUSABLE_TIMING => "该动作的时间信息无法被精确表示。",
        INGEST_MISSING_FILE => "所选文件已不存在。",
        INGEST_UNSUPPORTED_FORMAT => "该文件不是受支持的 FBX 来源。",
        "Inspecting Character…" => STATUS_INSPECTING_CHARACTER,
        "Inspecting Motion…" => STATUS_INSPECTING_MOTION,
        "Character source is ready." => STATUS_CHARACTER_READY,
        "Motion source is ready." => STATUS_MOTION_READY,
        "Multiple animation clips were found. Choose one clip." => STATUS_MULTI_CLIPS,
        "no Preview generated" => PREVIEW_NONE,
        "selection changed; previous Preview is not valid for this Product version" => {
            "选择已更改，先前预览不适用于当前产品版本。"
        }
        "Propose Mapping requires a selected Character version" => ERR_NO_CHARACTER,
        "Propose Mapping requires a selected Motion version" => ERR_NO_MOTION,
        "Compatibility requires a selected Character version" => ERR_NO_CHARACTER,
        "Compatibility requires a selected Motion version" => ERR_NO_MOTION,
        "Compatibility requires an accepted MappingVersion" => ERR_ACCEPT_MAPPING,
        "Workbench Transfer requires an exact CompatibilityResult" => ERR_NEED_COMPAT,
        "Transfer FAIL CLOSED: no Character version is selected" => ERR_NO_CHARACTER,
        "Transfer FAIL CLOSED: no Motion version is selected" => ERR_NO_MOTION,
        "Workbench has no Mapping to accept" => ERR_NO_MAPPING,
        "Workbench has no MappingVersion to accept" => ERR_NO_MAPPING,
        "Inspect a Character FBX before adding it." => ERR_INSPECT_CHARACTER,
        "Inspect a Motion FBX before adding it." => ERR_INSPECT_MOTION,
        "registration: display name is required" => ERR_DISPLAY_NAME,
        "Native Application path is required for Character registration" => ERR_NEED_APP_CHARACTER,
        "Native Application path is required for Motion registration" => ERR_NEED_APP_MOTION,
        "Native Application path is required for Mapping" => ERR_NEED_APP_MAPPING,
        "Native Application path is required for Compatibility" => ERR_NEED_APP_COMPAT,
        "Native Application path is required for Transfer" => ERR_NEED_APP_TRANSFER,
        "Mapping draft stored; explicit accept is required" => MSG_MAPPING_DRAFT,
        "Mapping accepted" => MSG_MAPPING_ACCEPTED,
        other if other.starts_with(INGEST_INSPECT_FAILED) => {
            return Some(format!(
                "无法用当前处理后端分析该 FBX。{}",
                other.trim_start_matches(INGEST_INSPECT_FAILED)
            ));
        }
        _ => return None,
    };
    Some(mapped.to_string())
}
