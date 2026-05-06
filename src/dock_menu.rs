#[cfg(not(target_os = "macos"))]
pub fn install() {}

#[cfg(not(target_os = "macos"))]
pub fn poll_action() -> u8 {
    0
}

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(target_os = "macos")]
static DOCK_ACTION: AtomicU8 = AtomicU8::new(0);

#[cfg(target_os = "macos")]
const ACTION_NEW_CONN: u8 = 2;

#[cfg(target_os = "macos")]
pub fn poll_action() -> u8 {
    DOCK_ACTION.swap(0, Ordering::SeqCst)
}

#[cfg(target_os = "macos")]
pub fn install() {
    use objc2::ffi;
    use objc2::runtime::{AnyObject, Imp, Sel};

    unsafe {
        register_handler_class();

        let app: *mut AnyObject = objc2::msg_send![objc2::class!(NSApplication), sharedApplication];
        if app.is_null() {
            return;
        }
        let delegate: *mut AnyObject = objc2::msg_send![app, delegate];
        if delegate.is_null() {
            return;
        }

        let cls = ffi::object_getClass(delegate);

        // applicationDockMenu: — Dock 우클릭 메뉴
        let dock_sel = ffi::sel_registerName(c"applicationDockMenu:".as_ptr()).unwrap();
        let dock_imp: Imp = std::mem::transmute(
            dock_menu_callback
                as unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject) -> *mut AnyObject,
        );
        ffi::class_addMethod(cls.cast_mut(), dock_sel, dock_imp, c"@@:@".as_ptr());

        // applicationShouldHandleReopen:hasVisibleWindows: — Dock 아이콘 클릭 시 창 복원
        let reopen_sel = ffi::sel_registerName(
            c"applicationShouldHandleReopen:hasVisibleWindows:".as_ptr(),
        )
        .unwrap();
        let reopen_imp: Imp = std::mem::transmute(
            reopen_callback
                as unsafe extern "C-unwind" fn(
                    *mut AnyObject,
                    Sel,
                    *mut AnyObject,
                    objc2::runtime::Bool,
                ) -> objc2::runtime::Bool,
        );
        ffi::class_addMethod(cls.cast_mut(), reopen_sel, reopen_imp, c"c@:@c".as_ptr());
    }
}

/// Dock 아이콘 클릭 시 호출 — 숨겨진 창을 네이티브 레벨에서 직접 복원.
#[cfg(target_os = "macos")]
unsafe extern "C-unwind" fn reopen_callback(
    _self: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    _app: *mut objc2::runtime::AnyObject,
    has_visible: objc2::runtime::Bool,
) -> objc2::runtime::Bool {
    if !has_visible.as_bool() {
        restore_main_window();
    }
    objc2::runtime::Bool::YES
}

#[cfg(target_os = "macos")]
unsafe fn restore_main_window() {
    use objc2::runtime::AnyObject;

    let app: *mut AnyObject = objc2::msg_send![objc2::class!(NSApplication), sharedApplication];
    let windows: *mut AnyObject = objc2::msg_send![app, windows];
    let count: usize = objc2::msg_send![windows, count];

    for i in 0..count {
        let window: *mut AnyObject = objc2::msg_send![windows, objectAtIndex: i];
        let _: () = objc2::msg_send![window, makeKeyAndOrderFront: std::ptr::null::<AnyObject>()];
    }

    let _: () = objc2::msg_send![app, activateIgnoringOtherApps: objc2::runtime::Bool::YES];
}

#[cfg(target_os = "macos")]
unsafe fn register_handler_class() {
    use objc2::ffi;
    use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};

    if AnyClass::get(c"FGDockHandler").is_some() {
        return;
    }

    let superclass = ffi::objc_getClass(c"NSObject".as_ptr());
    let cls = ffi::objc_allocateClassPair(superclass, c"FGDockHandler".as_ptr(), 0);
    if cls.is_null() {
        return;
    }

    let show_sel = ffi::sel_registerName(c"showWindow:".as_ptr()).unwrap();
    let show_imp: Imp = std::mem::transmute(
        show_window_action as unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject),
    );
    ffi::class_addMethod(cls, show_sel, show_imp, c"v@:@".as_ptr());

    let conn_sel = ffi::sel_registerName(c"newConnection:".as_ptr()).unwrap();
    let conn_imp: Imp = std::mem::transmute(
        new_connection_action as unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject),
    );
    ffi::class_addMethod(cls, conn_sel, conn_imp, c"v@:@".as_ptr());

    ffi::objc_registerClassPair(cls);
}

#[cfg(target_os = "macos")]
unsafe extern "C-unwind" fn show_window_action(
    _self: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    _sender: *mut objc2::runtime::AnyObject,
) {
    restore_main_window();
}

#[cfg(target_os = "macos")]
unsafe extern "C-unwind" fn new_connection_action(
    _self: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    _sender: *mut objc2::runtime::AnyObject,
) {
    restore_main_window();
    DOCK_ACTION.store(ACTION_NEW_CONN, Ordering::SeqCst);
}

#[cfg(target_os = "macos")]
unsafe extern "C-unwind" fn dock_menu_callback(
    _self: *mut objc2::runtime::AnyObject,
    _cmd: objc2::runtime::Sel,
    _app: *mut objc2::runtime::AnyObject,
) -> *mut objc2::runtime::AnyObject {
    use objc2::runtime::AnyObject;

    let menu: *mut AnyObject = objc2::msg_send![objc2::class!(NSMenu), new];
    let handler: *mut AnyObject = objc2::msg_send![objc2::class!(FGDockHandler), new];

    let empty: *mut AnyObject = objc2::msg_send![
        objc2::class!(NSString),
        stringWithUTF8String: c"".as_ptr()
    ];

    // "Show FerrumGrid"
    let show_title: *mut AnyObject = objc2::msg_send![
        objc2::class!(NSString),
        stringWithUTF8String: c"Show FerrumGrid".as_ptr()
    ];
    let show_sel = objc2::ffi::sel_registerName(c"showWindow:".as_ptr()).unwrap();
    let show_alloc: *mut AnyObject = objc2::msg_send![objc2::class!(NSMenuItem), alloc];
    let show_item: *mut AnyObject = objc2::msg_send![
        show_alloc,
        initWithTitle: show_title,
        action: show_sel,
        keyEquivalent: empty
    ];
    let _: () = objc2::msg_send![show_item, setTarget: handler];
    let _: () = objc2::msg_send![menu, addItem: show_item];

    // Separator
    let sep: *mut AnyObject = objc2::msg_send![objc2::class!(NSMenuItem), separatorItem];
    let _: () = objc2::msg_send![menu, addItem: sep];

    // "New Connection"
    let conn_title: *mut AnyObject = objc2::msg_send![
        objc2::class!(NSString),
        stringWithUTF8String: c"New Connection".as_ptr()
    ];
    let conn_sel = objc2::ffi::sel_registerName(c"newConnection:".as_ptr()).unwrap();
    let conn_alloc: *mut AnyObject = objc2::msg_send![objc2::class!(NSMenuItem), alloc];
    let conn_item: *mut AnyObject = objc2::msg_send![
        conn_alloc,
        initWithTitle: conn_title,
        action: conn_sel,
        keyEquivalent: empty
    ];
    let _: () = objc2::msg_send![conn_item, setTarget: handler];
    let _: () = objc2::msg_send![menu, addItem: conn_item];

    menu
}
