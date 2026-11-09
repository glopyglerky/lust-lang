//! Port of Go `internal/ls/hovericon.go` (tsgo#4625).

use crate::ls::prelude::*;

// Go: ls/hovericon.go:11 vsImageCatalogGuid
// vsImageCatalogGuid is the GUID of the shared VS image catalog (see
// Microsoft.VisualStudio.Imaging.KnownImageIds), mirroring the constant duplicated in
// TypeScript-VS's ImageIdMapping.cs (which avoids taking an assembly reference just for this GUID).
const VS_IMAGE_CATALOG_GUID: &str = "ae27a6b0-e345-4288-96df-5eaf394ee369";

// Go: ls/hovericon.go:16 imageId consts
// Known image IDs from Microsoft.VisualStudio.Imaging.KnownImageIds, restricted to the subset
// consumed by TypeScript-VS's ImageIdMapping.cs for hover tooltips. Corsa (this LSP server) has no
// TSServer/Roslyn dependency to reuse that mapping from, so the values are duplicated here.
const IMAGE_ID_WARNING: i32 = 0x00000637;
const IMAGE_ID_KEYWORD: i32 = 0x00000635;
const IMAGE_ID_MODULE_PRIVATE: i32 = 0x0000077D;
const IMAGE_ID_MODULE_PROTECTED: i32 = 0x0000077E;
const IMAGE_ID_MODULE_PUBLIC: i32 = 0x0000077F;
const IMAGE_ID_TYPE: i32 = 0x00000CA1;
const IMAGE_ID_NAMESPACE: i32 = 0x0000079F;
const IMAGE_ID_CLASS_PRIVATE: i32 = 0x000001D7;
const IMAGE_ID_CLASS_PROTECTED: i32 = 0x000001D8;
const IMAGE_ID_CLASS_PUBLIC: i32 = 0x000001D9;
const IMAGE_ID_INTERFACE_PRIVATE: i32 = 0x00000646;
const IMAGE_ID_INTERFACE_PROTECTED: i32 = 0x00000647;
const IMAGE_ID_INTERFACE_PUBLIC: i32 = 0x00000648;
const IMAGE_ID_ENUM_PRIVATE: i32 = 0x00000469;
const IMAGE_ID_ENUM_PROTECTED: i32 = 0x0000046A;
const IMAGE_ID_ENUM_PUBLIC: i32 = 0x0000046B;
const IMAGE_ID_ENUM_MEMBER: i32 = 0x00000465;
const IMAGE_ID_LOCAL_VARIABLE: i32 = 0x000006D3;
const IMAGE_ID_PROPERTY_PRIVATE: i32 = 0x00000982;
const IMAGE_ID_PROPERTY_PROTECTED: i32 = 0x00000983;
const IMAGE_ID_PROPERTY_PUBLIC: i32 = 0x00000984;
const IMAGE_ID_METHOD_PRIVATE: i32 = 0x00000756;
const IMAGE_ID_METHOD_PROTECTED: i32 = 0x00000757;
const IMAGE_ID_METHOD_PUBLIC: i32 = 0x00000758;
const IMAGE_ID_LABEL: i32 = 0x0000067D;
const IMAGE_ID_ASSEMBLY: i32 = 0x000000C4;
const IMAGE_ID_CONSTANT_PRIVATE: i32 = 0x0000026A;
const IMAGE_ID_CONSTANT_PROTECTED: i32 = 0x0000026B;
const IMAGE_ID_CONSTANT_PUBLIC: i32 = 0x0000026C;

// Go: ls/hovericon.go:48 newVSImageId
fn new_vs_image_id(id: i32) -> lsproto::VSImageId {
    lsproto::VSImageId {
        guid: VS_IMAGE_CATALOG_GUID.to_string(),
        id,
        ..Default::default()
    }
}

// Go: ls/hovericon.go:56 getVSHoverImageId
// getVSHoverImageId maps a symbol's ScriptElementKind/modifiers to the VS image shown next to the
// symbol name in hover tooltips. This mirrors TypeScript-VS's ImageIdMapping.GetImageId, which the
// legacy (TSServer-backed) hover path uses; Corsa has no TSServer to source that mapping from, so
// the LSP hover response must carry the equivalent icon directly.

// Go: ls/hovericon.go:132 buildVSHoverRawContent
// buildVSHoverRawContent assembles the VS-specific rich hover content (symbol icon + colorized
// declaration line, plus an optional colorized documentation block) matching the shape that
// TypeScript-VS's legacy HoverService.cs builds from TSServer's quickinfo-full response
// (ImageElement + ClassifiedTextElement wrapped in a ContainerElement).
// PORT: Go's runs are `[]*lsproto.VSClassifiedTextRun`; the port's runs are
// values, which are never nil.
pub fn build_vs_hover_raw_content(
    image_id: lsproto::VSImageId,
    quick_info_runs: Vec<lsproto::VSClassifiedTextRun>,
    documentation_runs: Vec<lsproto::VSClassifiedTextRun>,
) -> Option<lsproto::VSContainerElement> {
    if quick_info_runs.is_empty() {
        return None;
    }

    let display_line = lsproto::VSContainerElement {
        style: lsproto::VSContainerElementStyle::WRAPPED,
        elements: vec![
            lsproto::VSImageElementOrClassifiedTextElementOrContainerElement {
                image_element: Some(lsproto::VSImageElement {
                    image_id: Some(image_id),
                    ..Default::default()
                }),
                ..Default::default()
            },
            lsproto::VSImageElementOrClassifiedTextElementOrContainerElement {
                classified_text_element: Some(lsproto::VSClassifiedTextElement {
                    runs: quick_info_runs.into_iter().map(Some).collect(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    if documentation_runs.is_empty() {
        return Some(display_line);
    }

    Some(lsproto::VSContainerElement {
        style: lsproto::VSContainerElementStyle::STACKED,
        elements: vec![
            lsproto::VSImageElementOrClassifiedTextElementOrContainerElement {
                container_element: Some(display_line),
                ..Default::default()
            },
            lsproto::VSImageElementOrClassifiedTextElementOrContainerElement {
                classified_text_element: Some(lsproto::VSClassifiedTextElement {
                    runs: documentation_runs.into_iter().map(Some).collect(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        ..Default::default()
    })
}
