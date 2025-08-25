fn main() {
    // Compile resource files during build
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/resources.gresource.xml",
        "resources.gresource",
    );
}
