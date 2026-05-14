use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use amigo_scripting_api::ScriptSourceContext;
use rhai::{AST, Engine, EvalAltResult, Module, ModuleResolver, Position, Scope};
use serde::Deserialize;

use crate::bindings::WorldApi;

#[derive(Debug, Deserialize)]
struct ScriptPackageManifest {
    id: String,
    #[allow(dead_code)]
    version: String,
    #[serde(default)]
    exports: Vec<String>,
    #[serde(default)]
    requires: ScriptPackageRequirements,
}

#[derive(Debug, Default, Deserialize)]
struct ScriptPackageRequirements {
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Clone)]
pub struct PackageModuleResolver {
    engine_packages_root: PathBuf,
    source_context: Arc<Mutex<Option<ScriptSourceContext>>>,
    world: Option<WorldApi>,
}

impl Default for PackageModuleResolver {
    fn default() -> Self {
        Self {
            engine_packages_root: default_engine_packages_root(),
            source_context: Arc::new(Mutex::new(None)),
            world: None,
        }
    }
}

impl PackageModuleResolver {
    #[cfg(test)]
    pub fn new(engine_packages_root: impl Into<PathBuf>) -> Self {
        Self {
            engine_packages_root: engine_packages_root.into(),
            source_context: Arc::new(Mutex::new(None)),
            world: None,
        }
    }

    pub fn default_with_context(source_context: Arc<Mutex<Option<ScriptSourceContext>>>) -> Self {
        Self {
            engine_packages_root: default_engine_packages_root(),
            source_context,
            world: None,
        }
    }

    pub fn with_world(mut self, world: WorldApi) -> Self {
        self.world = Some(world);
        self
    }

    #[cfg(test)]
    pub fn set_source_context(&self, context: Option<ScriptSourceContext>) {
        *self
            .source_context
            .lock()
            .expect("script source context mutex should not be poisoned") = context;
    }

    fn resolve_path(&self, import: &str) -> Option<PathBuf> {
        if import.starts_with("pkg:") {
            return self.resolve_package_path(import);
        }
        if import.starts_with("mod:") {
            return self.resolve_mod_package_path(import);
        }
        if import.starts_with("./") || import.starts_with("../") {
            return self.resolve_relative_path(import);
        }
        None
    }

    fn resolve_package_path(&self, import: &str) -> Option<PathBuf> {
        let rest = import.strip_prefix("pkg:")?;
        if rest.contains("..") || rest.contains('\\') {
            return None;
        }
        let (package_id, module_name) = rest.rsplit_once('/')?;
        if module_name.is_empty() || package_id.is_empty() {
            return None;
        }

        let package_path = package_id
            .strip_prefix("amigo.")
            .unwrap_or(package_id)
            .replace('.', "/");
        let package_root = safe_join(&self.engine_packages_root, &PathBuf::from(package_path))?;
        let manifest = self.validated_manifest(&package_root, package_id, module_name)?;

        let mut relative = PathBuf::from(module_name);
        relative.set_extension("rhai");
        if !manifest
            .exports
            .iter()
            .any(|export| export == &display_path(&relative))
        {
            return None;
        }

        safe_join(&package_root, &relative)
    }

    fn resolve_mod_package_path(&self, import: &str) -> Option<PathBuf> {
        let rest = import.strip_prefix("mod:")?;
        if rest.contains("..") || rest.contains('\\') {
            return None;
        }
        let (package_id, module_name) = rest.split_once('/')?;
        if package_id.is_empty() || module_name.is_empty() {
            return None;
        }

        let context = self.current_context()?;
        let mut relative = PathBuf::from("scripts/packages");
        relative.push(package_id);
        let package_root = safe_join(&context.mod_root_path, &relative)?;
        let manifest = self.validated_manifest(&package_root, package_id, module_name)?;

        let mut module_relative = PathBuf::from(module_name);
        module_relative.set_extension("rhai");
        if !manifest
            .exports
            .iter()
            .any(|export| export == &display_path(&module_relative))
        {
            return None;
        }

        safe_join(&package_root, &module_relative)
    }

    fn resolve_relative_path(&self, import: &str) -> Option<PathBuf> {
        if import.contains('\\') {
            return None;
        }
        let context = self.current_context()?;
        let relative = Path::new(import);
        normalize_under_root(&context.mod_root_path, &context.script_dir_path, relative)
    }

    fn current_context(&self) -> Option<ScriptSourceContext> {
        self.source_context
            .lock()
            .expect("script source context mutex should not be poisoned")
            .clone()
    }

    fn validated_manifest(
        &self,
        package_root: &Path,
        package_id: &str,
        _module_name: &str,
    ) -> Option<ScriptPackageManifest> {
        let manifest_path = package_root.join("package.yml");
        let raw = std::fs::read_to_string(manifest_path).ok()?;
        let manifest = serde_yaml::from_str::<ScriptPackageManifest>(&raw).ok()?;
        if manifest.id != package_id {
            return None;
        }

        if let Some(world) = self.world.as_ref() {
            let available = world.runtime_capabilities();
            if !available.is_empty()
                && manifest
                    .requires
                    .capabilities
                    .iter()
                    .any(|required| !available.contains(required))
            {
                return None;
            }
        }

        Some(manifest)
    }

    fn compile_module(
        &self,
        engine: &Engine,
        import: &str,
        pos: Position,
    ) -> Result<rhai::Shared<Module>, Box<EvalAltResult>> {
        let file_path = self
            .resolve_path(import)
            .ok_or_else(|| Box::new(EvalAltResult::ErrorModuleNotFound(import.to_owned(), pos)))?;
        let mut ast = compile_file(engine, file_path, import, pos)?;
        ast.set_source(import);
        let mut scope = Scope::new();
        if let Some(world) = self.world.clone() {
            scope.push_constant("world", world);
        }
        let module = Module::eval_ast_as_new(scope, &ast, engine)?;
        Ok(module.into())
    }
}

impl ModuleResolver for PackageModuleResolver {
    fn resolve(
        &self,
        engine: &Engine,
        _source: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Result<rhai::Shared<Module>, Box<EvalAltResult>> {
        self.compile_module(engine, path, pos)
    }

    fn resolve_ast(
        &self,
        engine: &Engine,
        _source: Option<&str>,
        path: &str,
        pos: Position,
    ) -> Option<Result<AST, Box<EvalAltResult>>> {
        let file_path = self.resolve_path(path)?;
        Some(compile_file(engine, file_path, path, pos))
    }
}

fn compile_file(
    engine: &Engine,
    file_path: PathBuf,
    import: &str,
    pos: Position,
) -> Result<AST, Box<EvalAltResult>> {
    engine
        .compile_file(file_path)
        .map_err(|error| Box::new(EvalAltResult::ErrorInModule(import.to_owned(), error, pos)))
}

fn default_engine_packages_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for ancestor in manifest_dir.ancestors() {
        let candidate = ancestor.join("assets/scripts/amigo/packages");
        if candidate.is_dir() {
            return candidate;
        }
    }
    PathBuf::from("assets/scripts/amigo/packages")
}

fn safe_join(root: &Path, relative: &Path) -> Option<PathBuf> {
    if relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return None;
    }
    Some(root.join(relative))
}

fn display_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str().map(str::to_owned),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_under_root(root: &Path, base: &Path, relative: &Path) -> Option<PathBuf> {
    let candidate = base.join(relative);
    let mut normalized = PathBuf::new();
    for component in candidate.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(value) => normalized.push(value),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }

    if !normalized.starts_with(root) {
        return None;
    }
    if normalized.extension().is_none() {
        normalized.set_extension("rhai");
    }
    Some(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_import_resolves_amigo_package_path() {
        let resolver = PackageModuleResolver::new(default_engine_packages_root());
        let path = resolver
            .resolve_path("pkg:amigo.std/input")
            .expect("package path should resolve");
        assert!(path.ends_with("assets/scripts/amigo/packages/std/input.rhai"));
    }

    #[test]
    fn package_import_rejects_path_traversal() {
        let resolver = PackageModuleResolver::new(default_engine_packages_root());
        assert!(resolver.resolve_path("pkg:amigo.std/../secret").is_none());
    }

    #[test]
    fn mod_import_resolves_inside_current_mod_packages() {
        let resolver = PackageModuleResolver::new("assets/scripts/amigo/packages");
        let root = unique_test_root("mod_import_resolves_inside_current_mod_packages");
        let package_root = root.join("scripts/packages/editor_ext");
        std::fs::create_dir_all(&package_root).expect("test package directory should be created");
        std::fs::write(
            package_root.join("package.yml"),
            "id: editor_ext\nversion: 0.1.0\nexports:\n  - custom_presets.rhai\nrequires:\n  capabilities: []\n",
        )
        .expect("test package manifest should be written");
        resolver.set_source_context(Some(ScriptSourceContext {
            source_name: "scene:test:main".to_owned(),
            mod_root_path: root.clone(),
            script_dir_path: root.join("scenes/main"),
        }));

        let path = resolver
            .resolve_path("mod:editor_ext/custom_presets")
            .expect("mod package path should resolve");
        assert!(path.ends_with("scripts/packages/editor_ext/custom_presets.rhai"));
    }

    #[test]
    fn relative_import_resolves_inside_current_mod_root() {
        let resolver = PackageModuleResolver::new("assets/scripts/amigo/packages");
        resolver.set_source_context(Some(ScriptSourceContext {
            source_name: "scene:test:main".to_owned(),
            mod_root_path: PathBuf::from("mods/test-mod"),
            script_dir_path: PathBuf::from("mods/test-mod/scenes/main"),
        }));

        let path = resolver
            .resolve_path("./helpers/menu")
            .expect("relative path should resolve");
        assert!(path.ends_with("mods/test-mod/scenes/main/helpers/menu.rhai"));
    }

    #[test]
    fn package_import_rejects_modules_not_exported_by_manifest() {
        let resolver = PackageModuleResolver::new(default_engine_packages_root());
        assert!(resolver.resolve_path("pkg:amigo.std/missing").is_none());
    }

    fn unique_test_root(test_name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "amigo-rhai-package-{test_name}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        root
    }
}
