#include <godot_cpp/godot.hpp>
#include <godot_cpp/core/class_db.hpp>

#include "aetheris_scene.hpp"
#include "aetheris_avatar.hpp"
#include "aetheris_bus.hpp"
#include "aetheris_auth.hpp"
#include "aetheris_xr.hpp"
#include "aetheris_persist.hpp"

using namespace godot;

/// Initialize the Aetheris GDExtension library
extern "C" void GDE_EXPORT aetheris_library_init(godot::GDExtensionInterfaceGetProcAddress p_get_proc_address, const GDExtensionClassLibraryPtr p_library, GDExtensionInitialization *r_initialization) {
    godot::GDExtensionBinding::InitObject init_obj(p_get_proc_address, p_library, r_initialization);

    // Register all classes
    init_obj.register_class<AetherisScene>();
    init_obj.register_class<AetherisAvatar>();
    init_obj.register_class<AetherisBus>();
    init_obj.register_class<AetherisAuth>();
    init_obj.register_class<AetherisXR>();
    init_obj.register_class<AetherisMultiUser>();
    init_obj.register_class<AetherisPhysics>();
    init_obj.register_class<AetherisXRScene>();
    // P4-06-A4: Persistence and replay classes
    init_obj.register_class<AetherisPersist>();
    init_obj.register_class<AetherisRecord>();
    init_obj.register_class<AetherisReplay>();
    init_obj.register_class<AetherisBridge>();

    init_obj.register_initializer(initialize_aetheris_module);
    init_obj.register_terminator(uninitialize_aetheris_module);
    init_obj.set_minimum_library_initialization_level(MODULE_INITIALIZATION_LEVEL_SCENE);

    init_obj.init();
}

/// Initialize the Aetheris module
extern "C" void GDE_EXPORT initialize_aetheris_module(godot::ModuleInitializationLevel p_level) {
    if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
        return;
    }

    // Initialize any global resources or services here
    godot::UtilityFunctions::print("Aetheris GDExtension module initialized");
}

/// Uninitialize the Aetheris module
extern "C" void GDE_EXPORT uninitialize_aetheris_module(godot::ModuleInitializationLevel p_level) {
    if (p_level != MODULE_INITIALIZATION_LEVEL_SCENE) {
        return;
    }

    // Clean up any global resources or services here
    godot::UtilityFunctions::print("Aetheris GDExtension module uninitialized");
}
