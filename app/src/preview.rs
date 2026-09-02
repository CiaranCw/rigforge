//! Engine-independent Preview: derived, rebuildable, non-authoritative.
//!
//! Product truth validates Preview. Preview never validates Product truth.

use std::fs;
use std::path::{Path, PathBuf};

use rigforge_domain::{
    ContentDigest, DerivedVariantVersion, Lifecycle, LocationEvidence, PreviewArtifact, ProductKind,
    Validated, VerificationOutcome,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::AppError;
use crate::qc::sha256_file;
use crate::Application;

pub const PREVIEW_MEDIA_TYPE: &str = "model/gltf-binary";
pub const PREVIEW_RECIPE_VERSION: &str = "v1-6-preview-1";
pub const PREVIEW_GENERATOR_ID: &str = "rigforge-preview-generator/0.1.0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreviewSubject {
    Character { version_id: String },
    Motion { version_id: String },
    DerivedVariant { version_id: String },
}

impl PreviewSubject {
    pub fn kind(&self) -> ProductKind {
        match self {
            Self::Character { .. } => ProductKind::CharacterAssetVersion,
            Self::Motion { .. } => ProductKind::MotionAssetVersion,
            Self::DerivedVariant { .. } => ProductKind::DerivedVariantVersion,
        }
    }

    pub fn version_id(&self) -> &str {
        match self {
            Self::Character { version_id }
            | Self::Motion { version_id }
            | Self::DerivedVariant { version_id } => version_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreviewGenerationRequest {
    pub subject: PreviewSubject,
    pub synthetic: bool,
}

impl PreviewGenerationRequest {
    pub fn character(version_id: impl Into<String>) -> Self {
        Self {
            subject: PreviewSubject::Character {
                version_id: version_id.into(),
            },
            synthetic: false,
        }
    }

    pub fn motion(version_id: impl Into<String>) -> Self {
        Self {
            subject: PreviewSubject::Motion {
                version_id: version_id.into(),
            },
            synthetic: false,
        }
    }

    pub fn derived_variant(version_id: impl Into<String>) -> Self {
        Self {
            subject: PreviewSubject::DerivedVariant {
                version_id: version_id.into(),
            },
            synthetic: false,
        }
    }

    pub fn synthetic_motion(version_id: impl Into<String>) -> Self {
        Self {
            subject: PreviewSubject::Motion {
                version_id: version_id.into(),
            },
            synthetic: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PreviewGenerationJob {
    pub kind: ProductKind,
    pub product_version_id: String,
    pub source_path: PathBuf,
    pub expected_source_sha256: String,
    pub expected_source_size: u64,
    pub output_glb: PathBuf,
    pub clip_id: Option<String>,
    pub frame_start: Option<i64>,
    pub frame_end: Option<i64>,
    pub fps: Option<u32>,
    pub synthetic: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PreviewDescriptor {
    pub has_animation: bool,
    #[serde(default)]
    pub animation_names: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_animation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_s: Option<f64>,
    #[serde(default)]
    pub declared_losses: Vec<String>,
    pub generation_recipe: String,
    pub generator_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backend_build: Option<String>,
}

#[derive(Clone, Debug)]
pub struct GeneratedPreview {
    pub payload_path: PathBuf,
    pub descriptor: PreviewDescriptor,
}

pub trait PreviewGeneratorPort {
    fn generate(&self, job: &PreviewGenerationJob) -> Result<GeneratedPreview, AppError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewFailureKind {
    NoPreviewGenerated,
    GenerationFailed,
    MetadataMissing,
    ProductBindingMismatch,
    ProductVersionMismatch,
    PayloadMissing,
    PayloadSizeMismatch,
    PayloadDigestMismatch,
    UnsupportedMediaType,
    ViewerLoadFailure,
    AnimationMissing,
}

impl PreviewFailureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoPreviewGenerated => "no Preview generated",
            Self::GenerationFailed => "Preview generation failed",
            Self::MetadataMissing => "Preview metadata missing",
            Self::ProductBindingMismatch => "Product binding mismatch",
            Self::ProductVersionMismatch => "Product version mismatch",
            Self::PayloadMissing => "payload missing",
            Self::PayloadSizeMismatch => "payload size mismatch",
            Self::PayloadDigestMismatch => "payload digest mismatch",
            Self::UnsupportedMediaType => "unsupported media type",
            Self::ViewerLoadFailure => "viewer load failure",
            Self::AnimationMissing => "animation missing where required",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewFailure {
    pub kind: PreviewFailureKind,
    pub detail: String,
}

impl PreviewFailure {
    pub fn new(kind: PreviewFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }
}

/// Payload is present only after exact binding + size + SHA-256 succeed.
#[derive(Clone, Debug)]
pub struct PreviewView {
    pub subject: PreviewSubject,
    pub artifact: Option<Validated<PreviewArtifact>>,
    pub descriptor: Option<PreviewDescriptor>,
    pub payload: Option<Vec<u8>>,
    pub failure: Option<PreviewFailure>,
}

impl PreviewView {
    pub fn is_valid(&self) -> bool {
        self.failure.is_none() && self.payload.is_some()
    }
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn verify_source_before_generation(job: &PreviewGenerationJob) -> Result<(), AppError> {
    if job.synthetic {
        return Ok(());
    }
    if !job.source_path.is_file() {
        return Err(AppError::Catalog(format!(
            "Preview source missing: {}",
            job.source_path.display()
        )));
    }
    let meta = fs::metadata(&job.source_path)?;
    if meta.len() != job.expected_source_size {
        return Err(AppError::Catalog(format!(
            "Preview source size mismatch: expected {} found {}",
            job.expected_source_size,
            meta.len()
        )));
    }
    let found = sha256_file(&job.source_path)?;
    if found != job.expected_source_sha256 {
        return Err(AppError::Catalog(
            "Preview source digest mismatch; generation refused".into(),
        ));
    }
    Ok(())
}

/// Triangle (and optional translation animation) GLB. Test/fixture payload only.
pub fn synthetic_preview_glb(animated: bool) -> Vec<u8> {
    let mut bin = Vec::new();
    // positions: 3 vertices
    let positions: [f32; 9] = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
    bin.extend_from_slice(&bytemuck_f32(&positions));
    let pos_len = bin.len() as u32;
    let index_offset = align4(bin.len());
    while bin.len() < index_offset {
        bin.push(0);
    }
    let indices: [u16; 3] = [0, 1, 2];
    for i in indices {
        bin.extend_from_slice(&i.to_le_bytes());
    }
    let index_len = 6u32;
    let mut times_offset = 0u32;
    let mut trans_offset = 0u32;
    if animated {
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        times_offset = bin.len() as u32;
        for t in [0.0f32, 1.0f32] {
            bin.extend_from_slice(&t.to_le_bytes());
        }
        while bin.len() % 4 != 0 {
            bin.push(0);
        }
        trans_offset = bin.len() as u32;
        let keys: [f32; 6] = [0.0, 0.0, 0.0, 0.0, 0.35, 0.0];
        bin.extend_from_slice(&bytemuck_f32(&keys));
    }
    while bin.len() % 4 != 0 {
        bin.push(0);
    }
    let mut json = serde_json::json!({
        "asset": {"version": "2.0", "generator": PREVIEW_GENERATOR_ID},
        "scene": 0,
        "scenes": [{"nodes": [0]}],
        "nodes": [{"mesh": 0, "name": "preview_proxy"}],
        "meshes": [{"primitives": [{"attributes": {"POSITION": 0}, "indices": 1}]}],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5126,
                "count": 3,
                "type": "VEC3",
                "min": [0.0, 0.0, 0.0],
                "max": [1.0, 1.0, 0.0]
            },
            {
                "bufferView": 1,
                "componentType": 5123,
                "count": 3,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": pos_len},
            {"buffer": 0, "byteOffset": index_offset as u32, "byteLength": index_len, "target": 34963}
        ],
        "buffers": [{"byteLength": bin.len()}]
    });
    if animated {
        json["nodes"] = serde_json::json!([{
            "mesh": 0,
            "name": "preview_proxy",
            "translation": [0.0, 0.0, 0.0]
        }]);
        json["accessors"]
            .as_array_mut()
            .unwrap()
            .extend_from_slice(&[
                serde_json::json!({
                    "bufferView": 2,
                    "componentType": 5126,
                    "count": 2,
                    "type": "SCALAR",
                    "min": [0.0],
                    "max": [1.0]
                }),
                serde_json::json!({
                    "bufferView": 3,
                    "componentType": 5126,
                    "count": 2,
                    "type": "VEC3",
                    "min": [0.0, 0.0, 0.0],
                    "max": [0.0, 0.35, 0.0]
                }),
            ]);
        json["bufferViews"].as_array_mut().unwrap().extend_from_slice(&[
            serde_json::json!({"buffer": 0, "byteOffset": times_offset, "byteLength": 8}),
            serde_json::json!({"buffer": 0, "byteOffset": trans_offset, "byteLength": 24}),
        ]);
        json["animations"] = serde_json::json!([{
            "name": "proxy_motion",
            "samplers": [{"input": 2, "interpolation": "LINEAR", "output": 3}],
            "channels": [{"sampler": 0, "target": {"node": 0, "path": "translation"}}]
        }]);
    }
    let json_bytes = serde_json::to_vec(&json).expect("gltf json");
    pack_glb(&json_bytes, &bin)
}

fn bytemuck_f32(values: &[f32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(values.len() * 4);
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

fn align4(n: usize) -> usize {
    (n + 3) & !3
}

fn pack_glb(json: &[u8], bin: &[u8]) -> Vec<u8> {
    let mut json_chunk = json.to_vec();
    while json_chunk.len() % 4 != 0 {
        json_chunk.push(b' ');
    }
    let mut bin_chunk = bin.to_vec();
    while bin_chunk.len() % 4 != 0 {
        bin_chunk.push(0);
    }
    let total = 12 + 8 + json_chunk.len() + 8 + bin_chunk.len();
    let mut out = Vec::with_capacity(total);
    out.extend_from_slice(&0x46546C67u32.to_le_bytes()); // glTF
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json_chunk.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // JSON
    out.extend_from_slice(&json_chunk);
    out.extend_from_slice(&(bin_chunk.len() as u32).to_le_bytes());
    out.extend_from_slice(&0x004E4942u32.to_le_bytes()); // BIN
    out.extend_from_slice(&bin_chunk);
    out
}

pub struct MemoryPreviewGenerator {
    pub fail: bool,
    pub animated: bool,
}

impl Default for MemoryPreviewGenerator {
    fn default() -> Self {
        Self {
            fail: false,
            animated: true,
        }
    }
}

impl PreviewGeneratorPort for MemoryPreviewGenerator {
    fn generate(&self, job: &PreviewGenerationJob) -> Result<GeneratedPreview, AppError> {
        verify_source_before_generation(job)?;
        if self.fail {
            return Err(AppError::Worker("memory Preview generator failed".into()));
        }
        let animated = self.animated
            && job.kind != ProductKind::CharacterAssetVersion;
        let bytes = synthetic_preview_glb(animated);
        if let Some(parent) = job.output_glb.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&job.output_glb, &bytes)?;
        let descriptor = PreviewDescriptor {
            has_animation: animated,
            animation_names: if animated {
                vec!["proxy_motion".into()]
            } else {
                Vec::new()
            },
            default_animation: animated.then(|| "proxy_motion".into()),
            duration_s: animated.then_some(1.0),
            declared_losses: vec![
                "synthetic Preview payload; not a lossless export".into(),
                "materials/textures/lighting simplified".into(),
            ],
            generation_recipe: PREVIEW_RECIPE_VERSION.into(),
            generator_id: PREVIEW_GENERATOR_ID.into(),
            backend_kind: Some("memory".into()),
            backend_version: Some("test".into()),
            backend_build: None,
        };
        Ok(GeneratedPreview {
            payload_path: job.output_glb.clone(),
            descriptor,
        })
    }
}

impl Application {
    pub fn preview_root(&self) -> PathBuf {
        self.catalog.preview_root().to_path_buf()
    }

    pub fn generate_preview(
        &mut self,
        request: PreviewGenerationRequest,
    ) -> Result<Validated<PreviewArtifact>, AppError> {
        self.generate_preview_with(&crate::pinned_preview::BlenderPreviewGenerator, request)
    }

    pub fn generate_preview_with<G: PreviewGeneratorPort + ?Sized>(
        &mut self,
        generator: &G,
        request: PreviewGenerationRequest,
    ) -> Result<Validated<PreviewArtifact>, AppError> {
        let job = self.build_generation_job(&request.subject, request.synthetic)?;
        let generated = generator.generate(&job)?;
        self.persist_generated_preview(&request.subject, generated)
    }

    pub fn regenerate_preview_with<G: PreviewGeneratorPort + ?Sized>(
        &mut self,
        generator: &G,
        request: PreviewGenerationRequest,
    ) -> Result<Validated<PreviewArtifact>, AppError> {
        self.generate_preview_with(generator, request)
    }

    pub fn delete_preview(&mut self, preview_id: &str) -> Result<(), AppError> {
        self.catalog.delete_preview_artifact(preview_id)
    }

    pub fn resolve_preview_for_display(&self, subject: PreviewSubject) -> PreviewView {
        match self.resolve_preview_inner(&subject) {
            Ok(view) => view,
            Err(failure) => PreviewView {
                subject,
                artifact: None,
                descriptor: None,
                payload: None,
                failure: Some(failure),
            },
        }
    }

    fn resolve_preview_inner(&self, subject: &PreviewSubject) -> Result<PreviewView, PreviewFailure> {
        let artifact = self
            .catalog
            .latest_preview_for_exact_version(subject.kind(), subject.version_id())
            .map_err(|e| PreviewFailure::new(PreviewFailureKind::MetadataMissing, e.to_string()))?
            .ok_or_else(|| {
                PreviewFailure::new(
                    PreviewFailureKind::NoPreviewGenerated,
                    "no PreviewArtifact for this exact Product version",
                )
            })?;
        let record = artifact.as_record();
        let bound = record.bound_product_version_id().ok_or_else(|| {
            PreviewFailure::new(
                PreviewFailureKind::ProductBindingMismatch,
                "PreviewArtifact has no bound Product version",
            )
        })?;
        if bound != subject.version_id() {
            return Err(PreviewFailure::new(
                PreviewFailureKind::ProductVersionMismatch,
                format!("selected {} preview bound {}", subject.version_id(), bound),
            ));
        }
        match subject.kind() {
            ProductKind::CharacterAssetVersion => record
                .assert_bound_character(
                    rigforge_domain::CharacterAssetVersionId::parse(subject.version_id()).map_err(
                        |e| PreviewFailure::new(PreviewFailureKind::ProductBindingMismatch, e.to_string()),
                    )?,
                )
                .map_err(|e| {
                    PreviewFailure::new(PreviewFailureKind::ProductBindingMismatch, e.to_string())
                })?,
            ProductKind::MotionAssetVersion => record
                .assert_bound_motion(
                    rigforge_domain::MotionAssetVersionId::parse(subject.version_id()).map_err(
                        |e| PreviewFailure::new(PreviewFailureKind::ProductBindingMismatch, e.to_string()),
                    )?,
                )
                .map_err(|e| {
                    PreviewFailure::new(PreviewFailureKind::ProductBindingMismatch, e.to_string())
                })?,
            ProductKind::DerivedVariantVersion => record
                .assert_bound_derived_variant(
                    rigforge_domain::DerivedVariantVersionId::parse(subject.version_id()).map_err(
                        |e| PreviewFailure::new(PreviewFailureKind::ProductBindingMismatch, e.to_string()),
                    )?,
                )
                .map_err(|e| {
                    PreviewFailure::new(PreviewFailureKind::ProductBindingMismatch, e.to_string())
                })?,
        }
        if record.media_type() != PREVIEW_MEDIA_TYPE {
            return Err(PreviewFailure::new(
                PreviewFailureKind::UnsupportedMediaType,
                record.media_type(),
            ));
        }
        let location = record.location().ok_or_else(|| {
            PreviewFailure::new(PreviewFailureKind::PayloadMissing, "no location evidence")
        })?;
        let path = PathBuf::from(location.value());
        if !path.is_file() {
            return Err(PreviewFailure::new(
                PreviewFailureKind::PayloadMissing,
                path.display().to_string(),
            ));
        }
        let bytes = fs::read(&path).map_err(|e| {
            PreviewFailure::new(PreviewFailureKind::PayloadMissing, e.to_string())
        })?;
        if bytes.len() as u64 != record.size_bytes() {
            return Err(PreviewFailure::new(
                PreviewFailureKind::PayloadSizeMismatch,
                format!("expected {} found {}", record.size_bytes(), bytes.len()),
            ));
        }
        let digest = sha256_bytes(&bytes);
        if digest != record.digest().sha256() {
            return Err(PreviewFailure::new(
                PreviewFailureKind::PayloadDigestMismatch,
                "SHA-256 does not match PreviewArtifact.digest",
            ));
        }
        if subject.kind() != ProductKind::CharacterAssetVersion {
            let descriptor = read_descriptor(&path);
            if let Some(desc) = &descriptor {
                if !desc.has_animation {
                    return Err(PreviewFailure::new(
                        PreviewFailureKind::AnimationMissing,
                        "Motion/Derived Preview requires animation",
                    ));
                }
            }
            return Ok(PreviewView {
                subject: subject.clone(),
                artifact: Some(artifact),
                descriptor,
                payload: Some(bytes),
                failure: None,
            });
        }
        Ok(PreviewView {
            subject: subject.clone(),
            artifact: Some(artifact),
            descriptor: read_descriptor(&path),
            payload: Some(bytes),
            failure: None,
        })
    }

    fn persist_generated_preview(
        &mut self,
        subject: &PreviewSubject,
        generated: GeneratedPreview,
    ) -> Result<Validated<PreviewArtifact>, AppError> {
        let bytes = fs::read(&generated.payload_path)?;
        let digest = ContentDigest::parse(&sha256_bytes(&bytes))?;
        let producer = self.ensure_preview_producer(&generated.descriptor)?;
        let mut artifact = match subject {
            PreviewSubject::Character { version_id } => PreviewArtifact::for_character(
                rigforge_domain::CharacterAssetVersionId::parse(version_id)?,
                digest,
                bytes.len() as u64,
                PREVIEW_MEDIA_TYPE,
                producer,
            )?,
            PreviewSubject::Motion { version_id } => PreviewArtifact::for_motion(
                rigforge_domain::MotionAssetVersionId::parse(version_id)?,
                digest,
                bytes.len() as u64,
                PREVIEW_MEDIA_TYPE,
                producer,
            )?,
            PreviewSubject::DerivedVariant { version_id } => PreviewArtifact::for_derived_variant(
                rigforge_domain::DerivedVariantVersionId::parse(version_id)?,
                digest,
                bytes.len() as u64,
                PREVIEW_MEDIA_TYPE,
                producer,
            )?,
        };
        let dest = self
            .catalog
            .preview_root()
            .join(artifact.id().canonical())
            .join("preview.glb");
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&generated.payload_path, &dest)?;
        let copied = sha256_file(&dest)?;
        if copied != artifact.digest().sha256() {
            return Err(AppError::Catalog(
                "copied Preview payload digest mismatch".into(),
            ));
        }
        write_descriptor(&dest, &generated.descriptor)?;
        artifact.attach_location(LocationEvidence::filesystem_path(dest.display().to_string())?)?;
        self.catalog.persist_preview_artifact(&artifact, &dest)
    }

    fn ensure_preview_producer(
        &mut self,
        descriptor: &PreviewDescriptor,
    ) -> Result<rigforge_domain::BackendExecutionContextId, AppError> {
        let ctx = rigforge_domain::BackendExecutionContext::new(
            descriptor.backend_kind.clone().unwrap_or_else(|| "preview".into()),
            descriptor
                .backend_version
                .clone()
                .unwrap_or_else(|| "unspecified".into()),
            descriptor
                .backend_build
                .clone()
                .unwrap_or_else(|| "none".into()),
            descriptor.generator_id.clone(),
            descriptor.generation_recipe.clone(),
        )?;
        let id = ctx.id();
        self.catalog.put_validated(&Validated::certify(ctx)?)?;
        Ok(id)
    }

    fn build_generation_job(
        &self,
        subject: &PreviewSubject,
        synthetic: bool,
    ) -> Result<PreviewGenerationJob, AppError> {
        let out = std::env::temp_dir()
            .join(format!("rigforge-preview-{}", uuid::Uuid::now_v7()))
            .join("preview.glb");
        if synthetic {
            if !matches!(subject, PreviewSubject::Motion { .. }) {
                return Err(AppError::Catalog(
                    "synthetic Preview is only defined for Motion proxy architecture tests".into(),
                ));
            }
            let version_id = subject.version_id().to_string();
            let version = self.catalog.load_motion_version(&version_id)?;
            let _skeleton = self.catalog.load_source_skeleton(
                &version.as_record().source_skeleton_ref_id().canonical(),
            )?;
            let time = version.as_record().time();
            return Ok(PreviewGenerationJob {
                kind: ProductKind::MotionAssetVersion,
                product_version_id: version_id,
                source_path: PathBuf::from("synthetic-non-humanoid"),
                expected_source_sha256: String::new(),
                expected_source_size: 0,
                output_glb: out,
                clip_id: Some("synthetic_proxy".into()),
                frame_start: Some(time.start().value_num()),
                frame_end: Some(time.end().value_num()),
                fps: time.start().fps_num(),
                synthetic: true,
            });
        }
        match subject {
            PreviewSubject::Character { version_id } => {
                let version = self.catalog.load_character_version(version_id)?;
                let src = version.as_record().source();
                let path = crate::dispatch::filesystem_location(src)?;
                Ok(PreviewGenerationJob {
                    kind: ProductKind::CharacterAssetVersion,
                    product_version_id: version_id.clone(),
                    source_path: path,
                    expected_source_sha256: src.digest().sha256().to_string(),
                    expected_source_size: src.size_bytes(),
                    output_glb: out,
                    clip_id: None,
                    frame_start: None,
                    frame_end: None,
                    fps: None,
                    synthetic: false,
                })
            }
            PreviewSubject::Motion { version_id } => {
                let version = self.catalog.load_motion_version(version_id)?;
                let _skeleton = self
                    .catalog
                    .load_source_skeleton(&version.as_record().source_skeleton_ref_id().canonical())?;
                let src = version.as_record().source();
                let path = crate::dispatch::filesystem_location(src)?;
                let time = version.as_record().time();
                Ok(PreviewGenerationJob {
                    kind: ProductKind::MotionAssetVersion,
                    product_version_id: version_id.clone(),
                    source_path: path,
                    expected_source_sha256: src.digest().sha256().to_string(),
                    expected_source_size: src.size_bytes(),
                    output_glb: out,
                    clip_id: Some(time.clip_identity_evidence().to_string()),
                    frame_start: Some(time.start().value_num()),
                    frame_end: Some(time.end().value_num()),
                    fps: time.start().fps_num(),
                    synthetic: false,
                })
            }
            PreviewSubject::DerivedVariant { version_id } => {
                let version = self.catalog.load_derived_variant_version(version_id)?;
                let record: &DerivedVariantVersion = version.as_record();
                if record.lifecycle() != Lifecycle::Published {
                    return Err(AppError::Catalog(
                        "Derived Preview requires a Published DerivedVariantVersion".into(),
                    ));
                }
                let persistence_id = record.persistence_artifact_id().ok_or_else(|| {
                    AppError::Catalog("Derived Preview requires a PersistenceArtifact".into())
                })?;
                let persistence = self
                    .catalog
                    .load_artifact_metadata(&persistence_id.canonical())?;
                let verification_id = record.persistence_verification_id().ok_or_else(|| {
                    AppError::Catalog("Derived Preview requires PersistenceVerification".into())
                })?;
                let verification = self
                    .catalog
                    .load_persistence_verification(&verification_id.canonical())?;
                if verification.as_record().fresh_reopen() != VerificationOutcome::Pass
                    || verification.as_record().structural_verification()
                        != VerificationOutcome::Pass
                {
                    return Err(AppError::Catalog(
                        "Derived Preview requires PASS PersistenceVerification".into(),
                    ));
                }
                let path = self.catalog.durable_artifact_path(persistence.as_record());
                Ok(PreviewGenerationJob {
                    kind: ProductKind::DerivedVariantVersion,
                    product_version_id: version_id.clone(),
                    source_path: path,
                    expected_source_sha256: persistence.as_record().digest().sha256().to_string(),
                    expected_source_size: persistence.as_record().size_bytes(),
                    output_glb: out,
                    clip_id: None,
                    frame_start: None,
                    frame_end: None,
                    fps: None,
                    synthetic: false,
                })
            }
        }
    }
}

fn descriptor_path(payload: &Path) -> PathBuf {
    payload.with_file_name("descriptor.json")
}

fn write_descriptor(payload: &Path, descriptor: &PreviewDescriptor) -> Result<(), AppError> {
    let path = descriptor_path(payload);
    fs::write(path, serde_json::to_vec_pretty(descriptor).map_err(|e| {
        AppError::Catalog(format!("Preview descriptor serialize: {e}"))
    })?)?;
    Ok(())
}

fn read_descriptor(payload: &Path) -> Option<PreviewDescriptor> {
    let path = descriptor_path(payload);
    let bytes = fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// Viewer session written only after Application binding + integrity succeed.
/// Payload bytes are omitted when the view is invalid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreviewSessionDocument {
    pub selected_product_kind: String,
    pub selected_product_version_id: String,
    pub preview_artifact_id: Option<String>,
    pub producer_id: Option<String>,
    pub media_type: Option<String>,
    pub payload_sha256: Option<String>,
    pub payload_size: Option<u64>,
    pub binding_ok: bool,
    pub valid: bool,
    pub failure_kind: Option<String>,
    pub failure_detail: Option<String>,
    pub has_animation: Option<bool>,
    pub animation_names: Vec<String>,
    pub default_animation: Option<String>,
    pub duration_s: Option<f64>,
    pub declared_losses: Vec<String>,
    pub payload_filename: Option<String>,
}

pub struct PreviewSession {
    pub root: PathBuf,
    pub view: PreviewView,
    pub document: PreviewSessionDocument,
}

impl Application {
    pub fn materialize_preview_session(
        &self,
        subject: PreviewSubject,
    ) -> Result<PreviewSession, AppError> {
        let view = self.resolve_preview_for_display(subject);
        let root = std::env::temp_dir().join(format!(
            "rigforge-preview-session-{}",
            uuid::Uuid::now_v7()
        ));
        fs::create_dir_all(&root)?;
        let mut document = PreviewSessionDocument {
            selected_product_kind: match view.subject.kind() {
                ProductKind::CharacterAssetVersion => "CharacterAssetVersion".into(),
                ProductKind::MotionAssetVersion => "MotionAssetVersion".into(),
                ProductKind::DerivedVariantVersion => "DerivedVariantVersion".into(),
            },
            selected_product_version_id: view.subject.version_id().to_string(),
            preview_artifact_id: view
                .artifact
                .as_ref()
                .map(|a| a.as_record().id().canonical()),
            producer_id: view
                .artifact
                .as_ref()
                .map(|a| a.as_record().producer_id().canonical()),
            media_type: view
                .artifact
                .as_ref()
                .map(|a| a.as_record().media_type().to_string()),
            payload_sha256: view
                .artifact
                .as_ref()
                .map(|a| a.as_record().digest().sha256().to_string()),
            payload_size: view.artifact.as_ref().map(|a| a.as_record().size_bytes()),
            binding_ok: view.failure.as_ref().map(|f| {
                !matches!(
                    f.kind,
                    PreviewFailureKind::ProductBindingMismatch
                        | PreviewFailureKind::ProductVersionMismatch
                )
            }).unwrap_or(true),
            valid: view.is_valid(),
            failure_kind: view.failure.as_ref().map(|f| f.kind.as_str().to_string()),
            failure_detail: view.failure.as_ref().map(|f| f.detail.clone()),
            has_animation: view.descriptor.as_ref().map(|d| d.has_animation),
            animation_names: view
                .descriptor
                .as_ref()
                .map(|d| d.animation_names.clone())
                .unwrap_or_default(),
            default_animation: view
                .descriptor
                .as_ref()
                .and_then(|d| d.default_animation.clone()),
            duration_s: view.descriptor.as_ref().and_then(|d| d.duration_s),
            declared_losses: view
                .descriptor
                .as_ref()
                .map(|d| d.declared_losses.clone())
                .unwrap_or_default(),
            payload_filename: None,
        };
        if view.is_valid() {
            if let Some(bytes) = &view.payload {
                let payload_path = root.join("payload.glb");
                fs::write(&payload_path, bytes)?;
                document.payload_filename = Some("payload.glb".into());
            }
        }
        fs::write(
            root.join("session.json"),
            serde_json::to_vec_pretty(&document).map_err(|e| {
                AppError::Catalog(format!("Preview session serialize: {e}"))
            })?,
        )?;
        Ok(PreviewSession {
            root,
            view,
            document,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use crate::test_graph::{
        persist_core, persist_unpublished_without_authority, unpublished_graph,
    };
    use crate::Application;
    use rigforge_domain::{
        BackendExecutionContext, BackendExecutionContextId, CharacterAsset, CharacterAssetVersion,
        CharacterAssetVersionId, ContentDigest, LocationEvidence, MotionAsset, MotionAssetVersion,
        MotionAssetVersionId, PreviewArtifact, SourceArtifactEvidence, SourceSkeletonReference,
        TimeDomainProvenance, TimePoint, SamplingInterpretation, Validated,
    };

    fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
        static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "rf-v16-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }

    fn source_from_file(path: &Path) -> SourceArtifactEvidence {
        let bytes = fs::read(path).unwrap();
        SourceArtifactEvidence::new(
            LocationEvidence::filesystem_path(path.display().to_string()).unwrap(),
            ContentDigest::parse(&sha256_bytes(&bytes)).unwrap(),
            bytes.len() as u64,
            "application/octet-stream",
            Some("v1-6-test".into()),
            None,
        )
        .unwrap()
    }

    fn character_motion_app() -> (Application, String, String) {
        let mut app = Application::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut app.catalog, &g).unwrap();
        (
            app,
            g.character_version.id().canonical(),
            g.motion_version.id().canonical(),
        )
    }

    fn app_with_real_sources() -> (Application, String, String) {
        let character_src = temp_file("character.bin", b"character-source-bytes");
        let motion_src = temp_file("motion.bin", b"motion-source-bytes-xxxxxxxx");
        let mut app = Application::open_in_memory().unwrap();
        let mut character = CharacterAsset::new("Knight").unwrap();
        let mut character_version = CharacterAssetVersion::draft(
            character.id(),
            "Knight v1",
            source_from_file(&character_src),
        )
        .unwrap();
        character_version.publish().unwrap();
        character.bind_published(character_version.id());
        let source_skeleton = SourceSkeletonReference::new("UAL2").unwrap();
        let mut motion = MotionAsset::new("Walk Carry").unwrap();
        let mut motion_version = MotionAssetVersion::draft(
            motion.id(),
            "Walk Carry v1",
            source_skeleton.id(),
            TimeDomainProvenance::new(
                "clip:walk-carry",
                TimePoint::frames(1, 30, 1).unwrap(),
                TimePoint::frames(61, 30, 1).unwrap(),
                SamplingInterpretation::BakedEverySourceFrame,
                "unmapped target joints remain at target rest",
            )
            .unwrap(),
            source_from_file(&motion_src),
        )
        .unwrap();
        motion_version.publish().unwrap();
        motion.bind_published(motion_version.id());
        app.catalog
            .put_validated_pair(
                &Validated::certify(character.clone()).unwrap(),
                &Validated::certify(character_version.clone()).unwrap(),
            )
            .unwrap();
        app.catalog
            .put_validated(&Validated::certify(source_skeleton).unwrap())
            .unwrap();
        app.catalog
            .put_validated_pair(
                &Validated::certify(motion.clone()).unwrap(),
                &Validated::certify(motion_version.clone()).unwrap(),
            )
            .unwrap();
        (
            app,
            character_version.id().canonical(),
            motion_version.id().canonical(),
        )
    }

    fn glb_for(kind: &str, animated: bool) -> (ContentDigest, u64, PathBuf) {
        let path = temp_file(&format!("{kind}.glb"), &synthetic_preview_glb(animated));
        let digest = ContentDigest::parse(&sha256_file(&path).unwrap()).unwrap();
        let size = fs::metadata(&path).unwrap().len();
        (digest, size, path)
    }

    fn persist_backend(
        catalog: &mut crate::catalog::SqliteCatalog,
        backend: &BackendExecutionContext,
    ) {
        catalog
            .put_validated(&Validated::certify(backend.clone()).unwrap())
            .unwrap();
    }

    fn location_bearing_character_preview(
        version_id: CharacterAssetVersionId,
        producer: BackendExecutionContextId,
        tag: &str,
    ) -> (PreviewArtifact, PathBuf) {
        let (digest, size, path) = glb_for(tag, false);
        let mut artifact = PreviewArtifact::for_character(
            version_id,
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            producer,
        )
        .unwrap();
        artifact
            .attach_location(
                LocationEvidence::filesystem_path(path.display().to_string()).unwrap(),
            )
            .unwrap();
        (artifact, path)
    }

    fn assert_missing_producer(err: AppError) {
        let text = err.to_string();
        assert!(
            text.contains("producer_id") && text.contains("BackendExecutionContext"),
            "{text}"
        );
    }

    #[test]
    fn store_exact_character_preview() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (digest, size, path) = glb_for("character", false);
        let mut artifact = PreviewArtifact::for_character(
            g.character_version.id(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        let stored = catalog.persist_preview_artifact(&artifact, &path).unwrap();
        let loaded = catalog
            .load_preview_artifact(&stored.as_record().id().canonical())
            .unwrap();
        assert_eq!(
            loaded.as_record().bound_character_version_id(),
            Some(g.character_version.id())
        );
    }

    #[test]
    fn store_exact_motion_preview() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (digest, size, path) = glb_for("motion", true);
        let mut artifact = PreviewArtifact::for_motion(
            g.motion_version.id(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        catalog.persist_preview_artifact(&artifact, &path).unwrap();
    }

    #[test]
    fn persist_preview_artifact_rejects_nonexistent_producer() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (artifact, path) = location_bearing_character_preview(
            g.character_version.id(),
            BackendExecutionContextId::generate(),
            "missing-producer",
        );
        let err = catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap_err();
        assert_missing_producer(err);
        assert!(catalog
            .load_validated::<BackendExecutionContext>(&artifact.producer_id().canonical())
            .is_err());
    }

    #[test]
    fn put_validated_rejects_nonexistent_preview_producer() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (artifact, _path) = location_bearing_character_preview(
            g.character_version.id(),
            BackendExecutionContextId::generate(),
            "missing-producer-generic",
        );
        let err = catalog
            .put_validated(&Validated::certify(artifact.clone()).unwrap())
            .unwrap_err();
        assert_missing_producer(err);
        assert!(catalog
            .load_validated::<BackendExecutionContext>(&artifact.producer_id().canonical())
            .is_err());
    }

    #[test]
    fn persist_preview_artifact_accepts_existing_producer() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (artifact, path) = location_bearing_character_preview(
            g.character_version.id(),
            g.backend.id(),
            "existing-producer",
        );
        let stored = catalog.persist_preview_artifact(&artifact, &path).unwrap();
        assert_eq!(stored.as_record().producer_id(), g.backend.id());
        let loaded_producer = catalog
            .load_validated::<BackendExecutionContext>(&g.backend.id().canonical())
            .unwrap();
        assert_eq!(loaded_producer.as_record().id(), g.backend.id());
    }

    #[test]
    fn store_exact_derived_preview() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_unpublished_without_authority(&mut catalog, &g).unwrap();
        let (digest, size, path) = glb_for("derived", true);
        let mut artifact = PreviewArtifact::for_derived_variant(
            g.derived_version.id(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        catalog.persist_preview_artifact(&artifact, &path).unwrap();
    }

    #[test]
    fn wrong_product_binding_rejected_on_persist() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (digest, size, path) = glb_for("wrong", false);
        let foreign = CharacterAssetVersionId::generate();
        let mut artifact = PreviewArtifact::for_character(
            foreign,
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        let err = catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap_err();
        match err {
            AppError::NotFound { .. } | AppError::Catalog(_) => {}
            other => panic!("expected missing Product, got {other}"),
        }
    }

    #[test]
    fn missing_bound_product_rejected() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (digest, size, path) = glb_for("missing-derived", true);
        let mut artifact = PreviewArtifact::for_derived_variant(
            g.derived_version.id(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        assert!(catalog.persist_preview_artifact(&artifact, &path).is_err());
    }

    #[test]
    fn derived_preview_for_nonexistent_version_rejected() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (digest, size, path) = glb_for("ghost", true);
        let mut artifact = PreviewArtifact::for_derived_variant(
            rigforge_domain::DerivedVariantVersionId::generate(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        assert!(catalog.persist_preview_artifact(&artifact, &path).is_err());
    }

    #[test]
    fn payload_digest_mismatch_rejected() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (_digest, size, path) = glb_for("digest", false);
        let mut artifact = PreviewArtifact::for_character(
            g.character_version.id(),
            ContentDigest::parse(&"ab".repeat(32)).unwrap(),
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        let err = catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap_err();
        assert!(err.to_string().contains("digest"), "{err}");
    }

    #[test]
    fn payload_size_mismatch_rejected() {
        let mut catalog = crate::catalog::SqliteCatalog::open_in_memory().unwrap();
        let g = unpublished_graph();
        persist_core(&mut catalog, &g).unwrap();
        let (digest, _size, path) = glb_for("size", false);
        let mut artifact = PreviewArtifact::for_character(
            g.character_version.id(),
            digest,
            1,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        let err = catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap_err();
        assert!(err.to_string().contains("size"), "{err}");
    }

    #[test]
    fn resolve_preview_for_exact_selected_product() {
        let (mut app, character_id, _) = character_motion_app();
        let g = unpublished_graph();
        persist_backend(&mut app.catalog, &g.backend);
        let (digest, size, path) = glb_for("resolve", false);
        let mut artifact = PreviewArtifact::for_character(
            rigforge_domain::CharacterAssetVersionId::parse(&character_id).unwrap(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        app.catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap();
        let view = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: character_id,
        });
        assert!(view.is_valid(), "{:?}", view.failure);
        assert!(view.payload.is_some());
    }

    #[test]
    fn stale_preview_not_resolved_for_new_product_version() {
        let (mut app, character_id, _) = character_motion_app();
        let g = unpublished_graph();
        persist_backend(&mut app.catalog, &g.backend);
        let (digest, size, path) = glb_for("stale", false);
        let mut artifact = PreviewArtifact::for_character(
            rigforge_domain::CharacterAssetVersionId::parse(&character_id).unwrap(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        app.catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap();
        let other = CharacterAssetVersionId::generate().canonical();
        let view = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: other,
        });
        assert!(!view.is_valid());
        assert!(view.payload.is_none());
        assert!(matches!(
            view.failure.as_ref().map(|f| f.kind),
            Some(PreviewFailureKind::NoPreviewGenerated)
                | Some(PreviewFailureKind::ProductVersionMismatch)
                | Some(PreviewFailureKind::ProductBindingMismatch)
        ));
    }

    #[test]
    fn preview_deletion_and_regeneration_leaves_product_unchanged() {
        let (mut app, character_id, _) = app_with_real_sources();
        let before = app
            .catalog
            .load_character_version(&character_id)
            .unwrap()
            .as_record()
            .clone();
        let first = app
            .generate_preview_with(
                &MemoryPreviewGenerator::default(),
                PreviewGenerationRequest::character(&character_id),
            )
            .unwrap();
        let first_id = first.as_record().id().canonical();
        app.delete_preview(&first_id).unwrap();
        let after_delete = app
            .catalog
            .load_character_version(&character_id)
            .unwrap()
            .as_record()
            .clone();
        assert_eq!(before, after_delete);
        let second = app
            .regenerate_preview_with(
                &MemoryPreviewGenerator::default(),
                PreviewGenerationRequest::character(&character_id),
            )
            .unwrap();
        assert_ne!(first_id, second.as_record().id().canonical());
        let after_regen = app
            .catalog
            .load_character_version(&character_id)
            .unwrap()
            .as_record()
            .clone();
        assert_eq!(before, after_regen);
        let view = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: character_id,
        });
        assert!(view.is_valid());
        assert_eq!(
            view.artifact.as_ref().unwrap().as_record().id(),
            second.as_record().id()
        );
    }

    #[test]
    fn database_reopen_retains_preview_metadata() {
        let dir = std::env::temp_dir().join(format!("rf-v16-reopen-{}", uuid::Uuid::now_v7()));
        fs::create_dir_all(&dir).unwrap();
        let db = dir.join("catalog.sqlite");
        let character_id;
        let preview_id;
        let producer_id;
        {
            let mut app = Application::open(&db).unwrap();
            let g = unpublished_graph();
            persist_core(&mut app.catalog, &g).unwrap();
            character_id = g.character_version.id().canonical();
            producer_id = g.backend.id().canonical();
            let (digest, size, path) = glb_for("reopen", false);
            let mut artifact = PreviewArtifact::for_character(
                g.character_version.id(),
                digest,
                size,
                PREVIEW_MEDIA_TYPE,
                g.backend.id(),
            )
            .unwrap();
            artifact
                .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
                .unwrap();
            let stored = app.catalog.persist_preview_artifact(&artifact, &path).unwrap();
            preview_id = stored.as_record().id().canonical();
            assert_eq!(stored.as_record().producer_id().canonical(), producer_id);
        }
        let reopened = Application::open(&db).unwrap();
        let loaded = reopened
            .catalog
            .load_preview_artifact(&preview_id)
            .unwrap();
        assert_eq!(
            loaded.as_record().bound_product_version_id().as_deref(),
            Some(character_id.as_str())
        );
        assert_eq!(loaded.as_record().producer_id().canonical(), producer_id);
        let retained = reopened
            .catalog
            .load_validated::<BackendExecutionContext>(&producer_id)
            .unwrap();
        assert_eq!(retained.as_record().id().canonical(), producer_id);
    }

    #[test]
    fn source_digest_verified_before_generation() {
        let (mut app, character_id, _) = app_with_real_sources();
        let version = app.catalog.load_character_version(&character_id).unwrap();
        let path = PathBuf::from(version.as_record().source().location().value());
        fs::write(&path, b"tampered-source").unwrap();
        let err = app
            .generate_preview_with(
                &MemoryPreviewGenerator::default(),
                PreviewGenerationRequest::character(&character_id),
            )
            .unwrap_err();
        assert!(
            err.to_string().contains("digest") || err.to_string().contains("size"),
            "{err}"
        );
    }

    #[test]
    fn character_generation_strips_unintended_animation() {
        let (mut app, character_id, _) = app_with_real_sources();
        app.generate_preview_with(
            &MemoryPreviewGenerator {
                fail: false,
                animated: true,
            },
            PreviewGenerationRequest::character(&character_id),
        )
        .unwrap();
        let view = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: character_id,
        });
        assert!(view.is_valid());
        let desc = view.descriptor.unwrap();
        assert!(!desc.has_animation);
        assert!(desc.animation_names.is_empty());
    }

    #[test]
    fn motion_generation_does_not_require_target_character() {
        let motion_src = temp_file("motion-only.bin", b"motion-only-source-bytes");
        let mut app = Application::open_in_memory().unwrap();
        let source_skeleton = SourceSkeletonReference::new("probe-skel").unwrap();
        let mut motion = MotionAsset::new("Probe Motion").unwrap();
        let mut motion_version = MotionAssetVersion::draft(
            motion.id(),
            "Probe v1",
            source_skeleton.id(),
            TimeDomainProvenance::new(
                "clip:probe",
                TimePoint::frames(1, 30, 1).unwrap(),
                TimePoint::frames(31, 30, 1).unwrap(),
                SamplingInterpretation::BakedEverySourceFrame,
                "unmapped target joints remain at target rest",
            )
            .unwrap(),
            source_from_file(&motion_src),
        )
        .unwrap();
        motion_version.publish().unwrap();
        motion.bind_published(motion_version.id());
        app.catalog
            .put_validated(&Validated::certify(source_skeleton).unwrap())
            .unwrap();
        app.catalog
            .put_validated_pair(
                &Validated::certify(motion).unwrap(),
                &Validated::certify(motion_version.clone()).unwrap(),
            )
            .unwrap();
        let stored = app
            .generate_preview_with(
                &MemoryPreviewGenerator::default(),
                PreviewGenerationRequest::motion(motion_version.id().canonical()),
            )
            .unwrap();
        assert_eq!(
            stored.as_record().bound_motion_version_id(),
            Some(motion_version.id())
        );
        assert!(stored.as_record().bound_character_version_id().is_none());
    }

    #[test]
    fn motion_generic_proxy_generated() {
        let (mut app, _, motion_id) = app_with_real_sources();
        app.generate_preview_with(
            &MemoryPreviewGenerator::default(),
            PreviewGenerationRequest::motion(&motion_id),
        )
        .unwrap();
        let view = app.resolve_preview_for_display(PreviewSubject::Motion {
            version_id: motion_id,
        });
        assert!(view.is_valid());
        let desc = view.descriptor.unwrap();
        assert!(desc.has_animation);
        assert!(desc.declared_losses.iter().any(|l| l.contains("simplified")));
    }

    #[test]
    fn generator_failure_leaves_product_unchanged() {
        let (mut app, character_id, _) = app_with_real_sources();
        let before = app
            .catalog
            .load_character_version(&character_id)
            .unwrap()
            .as_record()
            .clone();
        let err = app
            .generate_preview_with(
                &MemoryPreviewGenerator {
                    fail: true,
                    animated: false,
                },
                PreviewGenerationRequest::character(&character_id),
            )
            .unwrap_err();
        assert!(err.to_string().contains("failed"), "{err}");
        let after = app
            .catalog
            .load_character_version(&character_id)
            .unwrap()
            .as_record()
            .clone();
        assert_eq!(before, after);
        let view = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: character_id,
        });
        assert!(!view.is_valid());
        assert_eq!(
            view.failure.as_ref().unwrap().kind,
            PreviewFailureKind::NoPreviewGenerated
        );
    }

    #[test]
    fn preview_generation_creates_new_derivative_identity() {
        let (mut app, character_id, _) = app_with_real_sources();
        let a = app
            .generate_preview_with(
                &MemoryPreviewGenerator::default(),
                PreviewGenerationRequest::character(&character_id),
            )
            .unwrap();
        let b = app
            .generate_preview_with(
                &MemoryPreviewGenerator::default(),
                PreviewGenerationRequest::character(&character_id),
            )
            .unwrap();
        assert_ne!(a.as_record().id(), b.as_record().id());
    }

    #[test]
    fn binding_mismatch_blocks_display() {
        let (mut app, character_id, motion_id) = character_motion_app();
        let g = unpublished_graph();
        persist_backend(&mut app.catalog, &g.backend);
        let (digest, size, path) = glb_for("mismatch", true);
        let mut artifact = PreviewArtifact::for_motion(
            MotionAssetVersionId::parse(&motion_id).unwrap(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        app.catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap();
        let view = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: character_id,
        });
        assert!(!view.is_valid());
        assert!(view.payload.is_none());
    }

    #[test]
    fn payload_digest_mismatch_blocks_display() {
        let (mut app, character_id, _) = character_motion_app();
        let g = unpublished_graph();
        persist_backend(&mut app.catalog, &g.backend);
        let (digest, size, path) = glb_for("display-digest", false);
        let mut artifact = PreviewArtifact::for_character(
            CharacterAssetVersionId::parse(&character_id).unwrap(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        app.catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap();
        let original = fs::read(&path).unwrap();
        let mut corrupted = original.clone();
        corrupted[0] ^= 0xff;
        fs::write(&path, corrupted).unwrap();
        let view = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: character_id,
        });
        assert!(!view.is_valid());
        assert!(view.payload.is_none());
        assert_eq!(
            view.failure.as_ref().unwrap().kind,
            PreviewFailureKind::PayloadDigestMismatch
        );
    }

    #[test]
    fn selection_change_cannot_leave_stale_previous_preview_marked_valid() {
        let (mut app, character_id, motion_id) = character_motion_app();
        let g = unpublished_graph();
        persist_backend(&mut app.catalog, &g.backend);
        let (digest, size, path) = glb_for("prev", false);
        let mut artifact = PreviewArtifact::for_character(
            CharacterAssetVersionId::parse(&character_id).unwrap(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        app.catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap();
        let first = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: character_id.clone(),
        });
        assert!(first.is_valid());
        let second = app.resolve_preview_for_display(PreviewSubject::Motion {
            version_id: motion_id,
        });
        assert!(!second.is_valid());
        assert!(second.payload.is_none());
        let session = app
            .materialize_preview_session(PreviewSubject::Motion {
                version_id: second.subject.version_id().to_string(),
            })
            .unwrap();
        assert!(!session.document.valid);
        assert!(session.document.payload_filename.is_none());
        assert!(!session.root.join("payload.glb").is_file());
    }

    #[test]
    fn missing_payload_rejected_on_display() {
        let (mut app, character_id, _) = character_motion_app();
        let g = unpublished_graph();
        persist_backend(&mut app.catalog, &g.backend);
        let (digest, size, path) = glb_for("missing-payload", false);
        let mut artifact = PreviewArtifact::for_character(
            CharacterAssetVersionId::parse(&character_id).unwrap(),
            digest,
            size,
            PREVIEW_MEDIA_TYPE,
            g.backend.id(),
        )
        .unwrap();
        artifact
            .attach_location(LocationEvidence::filesystem_path(path.display().to_string()).unwrap())
            .unwrap();
        app.catalog
            .persist_preview_artifact(&artifact, &path)
            .unwrap();
        fs::remove_file(&path).unwrap();
        let view = app.resolve_preview_for_display(PreviewSubject::Character {
            version_id: character_id,
        });
        assert!(!view.is_valid());
        assert_eq!(
            view.failure.as_ref().unwrap().kind,
            PreviewFailureKind::PayloadMissing
        );
    }

    #[test]
    fn synthetic_motion_does_not_require_target_character() {
        let (mut app, _, motion_id) = app_with_real_sources();
        let stored = app
            .generate_preview_with(
                &MemoryPreviewGenerator::default(),
                PreviewGenerationRequest::synthetic_motion(&motion_id),
            )
            .unwrap();
        assert!(stored.as_record().bound_character_version_id().is_none());
        let view = app.resolve_preview_for_display(PreviewSubject::Motion {
            version_id: motion_id,
        });
        assert!(view.is_valid());
        assert!(view.descriptor.unwrap().has_animation);
    }
}
