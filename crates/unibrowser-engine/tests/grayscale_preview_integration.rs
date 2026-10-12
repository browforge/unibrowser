use unibrowser_engine::{
    CapabilityResolver, ContributionMergeDiagnostics, KindHostCapabilityMatrix, LoadedModContribution,
    ResolutionContext, core_tab_layout, merge_mod_contributions,
};
use unibrowser_mod_api::{
    CapabilityState, CommandContext, HostContext, ModManifest, Mode,
};

fn grayscale_contributions() -> LoadedModContribution {
    let manifest: ModManifest =
        serde_json::from_str(include_str!("../../../mods/image-grayscale-preview/info.json"))
            .expect("manifest");
    LoadedModContribution {
        mod_name: manifest.name.clone(),
        contributes: manifest.contributes.unwrap_or_default(),
    }
}

#[test]
fn grayscale_control_merges_into_image_adjust_family() {
    let contributions = vec![grayscale_contributions()];
    let resolver = CapabilityResolver::default();
    let host = HostContext {
        mode: Mode::Image,
        doc_type: "image".into(),
        backend_id: "base-png".into(),
        selection: Default::default(),
        capabilities: Default::default(),
    };
    let ctx = ResolutionContext::new(CommandContext::Image, host, Some("demo.png".into()), 1);
    let mut families = core_tab_layout(&ctx, &resolver);
    let mut diag = ContributionMergeDiagnostics::default();
    merge_mod_contributions(&mut families, &contributions, &mut diag);
    let image = families
        .iter()
        .find(|f| f.family_id == "unibrowser.family.image-adjust")
        .expect("image-adjust");
    let ids: Vec<_> = image
        .groups
        .iter()
        .flat_map(|g| g.controls.iter())
        .map(|c| c.control_id.as_str())
        .collect();
    assert!(ids.contains(&"image-grayscale-preview.control.grayscale"));
}

#[test]
fn grayscale_matrix_supported_on_png_host() {
    let contributions = vec![grayscale_contributions()];
    let resolver = CapabilityResolver::default();
    let matrix = KindHostCapabilityMatrix::from_contributions(&contributions);
    let host = HostContext {
        mode: Mode::Image,
        doc_type: "image".into(),
        backend_id: "base-png".into(),
        selection: Default::default(),
        capabilities: Default::default(),
    };
    let ctx = ResolutionContext::new(CommandContext::Image, host, Some("demo.png".into()), 1);
    let cap = matrix
        .resolve_control(
            "image-grayscale-preview.control.grayscale",
            Some("image-grayscale-preview::native"),
            "unibrowser.family.image-adjust",
            &ctx,
        )
        .expect("matrix");
    assert!(matches!(cap, CapabilityState::Supported));
}
