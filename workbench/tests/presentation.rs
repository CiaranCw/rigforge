use rigforge_app::AssetListItem;
use rigforge_workbench::i18n;
use rigforge_workbench::{TransferPhase, WorkbenchApp};

#[test]
fn selected_character_uses_exact_list_display_name() {
    let mut shell = WorkbenchApp::empty();
    shell.bind_list_items_for_test(
        vec![AssetListItem {
            logical_id: "char-logical".into(),
            display_name: "Knight Male".into(),
            record_type: "character_asset".into(),
            published_version_id: Some("char-version-exact".into()),
            draft_version_id: None,
        }],
        Vec::new(),
        Vec::new(),
    );
    shell.select_character_version("char-version-exact");
    assert_eq!(shell.selected_character_display_name(), Some("Knight Male"));
    assert_ne!(
        shell.selected_character_display_name(),
        shell.selected_character_version()
    );
}

#[test]
fn selected_motion_uses_exact_list_display_name() {
    let mut shell = WorkbenchApp::empty();
    shell.bind_list_items_for_test(
        Vec::new(),
        vec![AssetListItem {
            logical_id: "motion-logical".into(),
            display_name: "UAL2 Standard".into(),
            record_type: "motion_asset".into(),
            published_version_id: Some("motion-version-exact".into()),
            draft_version_id: None,
        }],
        Vec::new(),
    );
    shell.select_motion_version("motion-version-exact");
    assert_eq!(
        shell.selected_motion_display_name(),
        Some("UAL2 Standard")
    );
    assert_ne!(
        shell.selected_motion_display_name(),
        shell.selected_motion_version()
    );
}

#[test]
fn display_name_does_not_guess_first_or_latest() {
    let mut shell = WorkbenchApp::empty();
    shell.bind_list_items_for_test(
        vec![
            AssetListItem {
                logical_id: "a".into(),
                display_name: "First Character".into(),
                record_type: "character_asset".into(),
                published_version_id: Some("version-a".into()),
                draft_version_id: None,
            },
            AssetListItem {
                logical_id: "b".into(),
                display_name: "Knight Male".into(),
                record_type: "character_asset".into(),
                published_version_id: Some("version-b".into()),
                draft_version_id: None,
            },
        ],
        Vec::new(),
        Vec::new(),
    );
    shell.select_character_version("version-b");
    assert_eq!(shell.selected_character_display_name(), Some("Knight Male"));
    shell.select_character_version("missing-version");
    assert_eq!(shell.selected_character_display_name(), None);
}

#[test]
fn add_motion_status_is_human_readable_without_uuid() {
    let status = i18n::motion_added_status("UAL2 Standard", "Sword Dash");
    assert_eq!(
        status,
        "动作已添加并设为当前选择：UAL2 Standard · Sword Dash"
    );
    assert!(!status.contains("MotionAssetVersion"));
    assert!(!status.contains("version_id"));
    let character = i18n::character_added_status("Knight Male");
    assert_eq!(character, "角色已添加并设为当前选择：Knight Male");
    assert!(!character.contains("version"));
}

#[test]
fn clip_friendly_label_uses_last_identity_segment() {
    assert_eq!(
        i18n::clip_friendly_label("Armature|Armature|Sword_Dash"),
        "Sword Dash"
    );
}

#[test]
fn chinese_progress_and_workflow_labels() {
    assert_eq!(TransferPhase::Launching.user_label(), "正在启动处理后端…");
    assert_eq!(
        TransferPhase::RunningExecute.user_label(),
        "正在执行动作迁移…"
    );
    assert_eq!(TransferPhase::ValidatingQc.user_label(), "正在验证迁移结果…");
    assert_eq!(
        TransferPhase::ValidatingPersistence.user_label(),
        "正在验证已保存结果…"
    );
    assert_eq!(
        TransferPhase::Publishing.user_label(),
        "正在发布派生资产…"
    );
    assert_eq!(TransferPhase::Complete.user_label(), "迁移完成");
    assert_eq!(TransferPhase::Failed.user_label(), "迁移失败");
    assert_eq!(i18n::BTN_PROPOSE_MAPPING, "生成映射");
    assert_eq!(i18n::BTN_ACCEPT_MAPPING, "接受映射");
    assert_eq!(i18n::BTN_EVALUATE_COMPAT, "检查兼容性");
    assert_eq!(i18n::BTN_TRANSFER, "执行迁移");
    assert_eq!(i18n::ACK_WARNINGS, "我已阅读并了解以上兼容性警告");
}

#[test]
fn chinese_error_labels_preserve_first_use_guidance() {
    assert_eq!(
        i18n::present_user_text(rigforge_app::INGEST_NO_SKELETON),
        "未在该 FBX 中发现可用骨架。"
    );
    assert_eq!(
        i18n::present_user_text(rigforge_app::INGEST_MULTIPLE_SKELETONS),
        "该 FBX 中发现多个可用骨架。当前版本要求每个来源文件只有一个明确骨架。"
    );
    assert_eq!(
        i18n::present_user_text(rigforge_app::INGEST_NO_CLIPS),
        "未找到可用动画片段。"
    );
    assert_eq!(
        i18n::present_user_text(rigforge_app::INGEST_FILE_CHANGED),
        "文件在分析后发生了变化，请重新分析后再添加。"
    );
    assert_eq!(
        i18n::present_user_text("Propose Mapping requires a selected Motion version"),
        "当前没有已选择的动作。"
    );
    assert_eq!(
        i18n::present_user_text("Compatibility requires an accepted MappingVersion"),
        "请先接受骨骼映射。"
    );
    assert_eq!(
        i18n::present_user_text("Workbench Transfer requires an exact CompatibilityResult"),
        "请先完成兼容性检查。"
    );
}

#[test]
fn ordinary_workflow_status_does_not_require_raw_ids() {
    let src = include_str!("../src/lib.rs");
    assert!(src.contains("i18n::CURRENT_SELECTION"));
    assert!(src.contains("i18n::TECHNICAL_DETAILS"));
    assert!(src.contains("selected_character_display_name"));
    assert!(src.contains("selected_motion_display_name"));
    assert!(!src.contains("Character version: {}"));
    assert!(src.contains("CollapsingHeader::new(i18n::MAPPING_DETAILS)"));
}
