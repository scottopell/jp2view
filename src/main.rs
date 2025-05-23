#![deny(unsafe_op_in_unsafe_fn)]
#![allow(non_snake_case)]

use std::cell::{OnceCell, RefCell};

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Bool, ProtocolObject};
use objc2::AnyThread;
use objc2::{define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSAutoresizingMaskOptions,
    NSBackingStoreType, NSBezelStyle, NSBitmapImageRep, NSButton, NSEvent, NSImage, NSImageScaling,
    NSImageView, NSMagnificationGestureRecognizer, NSScrollView, NSSlider, NSWindow,
    NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    ns_string, NSArray, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSURL,
};
use objc2_uniform_type_identifiers::UTType;

//------------------------------------------------------------------------------
// Bitmap Font Definition
//------------------------------------------------------------------------------
/// A simple 5x5 pixel bitmap font for rendering text in the image viewer
/// Each character is represented as a 5x5 grid of binary pixels (0 = transparent, 1 = filled)
/// The array contains 30 characters in the following order:
/// C, O, M, I, N, G, S, P, J, 2, (space), F, L, E, D, T, A, R, B, 0-9, -, .
const BITMAP_CHARS: [[[u8; 5]; 5]; 30] = [
    // 0: C
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [0, 1, 1, 1, 0],
    ],
    // 1: O
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
    ],
    // 2: M
    [
        [1, 0, 0, 0, 1],
        [1, 1, 0, 1, 1],
        [1, 0, 1, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
    ],
    // 3: I
    [
        [0, 1, 1, 1, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 1, 1, 1, 0],
    ],
    // 4: N
    [
        [1, 0, 0, 0, 1],
        [1, 1, 0, 0, 1],
        [1, 0, 1, 0, 1],
        [1, 0, 0, 1, 1],
        [1, 0, 0, 0, 1],
    ],
    // 5: G
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 0],
        [1, 0, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
    ],
    // 6: S
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 0],
        [0, 1, 1, 1, 0],
        [0, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
    ],
    // 7: P
    [
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
    ],
    // 8: J
    [
        [0, 0, 1, 1, 0],
        [0, 0, 0, 1, 0],
        [0, 0, 0, 1, 0],
        [1, 0, 0, 1, 0],
        [0, 1, 1, 0, 0],
    ],
    // 9: 2
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [0, 0, 1, 1, 0],
        [0, 1, 0, 0, 0],
        [1, 1, 1, 1, 1],
    ],
    // 10: SPACE
    [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ],
    // 11: F
    [
        [1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0],
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
    ],
    // 12: L
    [
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [1, 0, 0, 0, 0],
        [1, 1, 1, 1, 1],
    ],
    // 13: E
    [
        [1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0],
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 0],
        [1, 1, 1, 1, 1],
    ],
    // 14: D
    [
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 0],
    ],
    // 15: T
    [
        [1, 1, 1, 1, 1],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
    ],
    // 16: A
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 1],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
    ],
    // 17: R
    [
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 0],
        [1, 0, 1, 0, 0],
        [1, 0, 0, 1, 0],
    ],
    // 18: B
    [
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 0],
    ],
    // 19: 0
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
    ],
    // 20: 1
    [
        [0, 0, 1, 0, 0],
        [0, 1, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 0, 1, 0, 0],
        [0, 1, 1, 1, 0],
    ],
    // 21: 3
    [
        [0, 1, 1, 1, 0],
        [0, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
        [0, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
    ],
    // 22: 4
    [
        [1, 0, 0, 0, 1],
        [1, 0, 0, 0, 1],
        [1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1],
        [0, 0, 0, 0, 1],
    ],
    // 23: 5
    [
        [1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0],
        [1, 1, 1, 1, 0],
        [0, 0, 0, 0, 1],
        [1, 1, 1, 1, 0],
    ],
    // 24: 6
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 0],
        [1, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
    ],
    // 25: 7
    [
        [1, 1, 1, 1, 1],
        [0, 0, 0, 0, 1],
        [0, 0, 0, 1, 0],
        [0, 0, 1, 0, 0],
        [0, 1, 0, 0, 0],
    ],
    // 26: 8
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
    ],
    // 27: 9
    [
        [0, 1, 1, 1, 0],
        [1, 0, 0, 0, 1],
        [0, 1, 1, 1, 1],
        [0, 0, 0, 0, 1],
        [0, 1, 1, 1, 0],
    ],
    // 28: - (dash)
    [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [1, 1, 1, 1, 1],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ],
    // 29: . (period)
    [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 1, 0, 0],
    ],
];

/// Mapping from characters to their index in the BITMAP_CHARS array
/// Unknown characters will map to index 10 (space) as a fallback
const CHAR_INDICES: [(char, usize); 30] = [
    ('C', 0),
    ('O', 1),
    ('M', 2),
    ('I', 3),
    ('N', 4),
    ('G', 5),
    ('S', 6),
    ('P', 7),
    ('J', 8),
    ('2', 9),
    (' ', 10),
    ('F', 11),
    ('L', 12),
    ('E', 13),
    ('D', 14),
    ('T', 15),
    ('A', 16),
    ('R', 17),
    ('B', 18),
    ('0', 19),
    ('1', 20),
    ('3', 21),
    ('4', 22),
    ('5', 23),
    ('6', 24),
    ('7', 25),
    ('8', 26),
    ('9', 27),
    ('-', 28),
    ('.', 29),
];

// Structure to hold source pattern and debug pixel data
#[derive(Debug)]
struct SourcePattern {
    buffer: Vec<u8>,
    width: usize,
    height: usize,
    bytes_per_row: usize,
}

// Enum to represent different pattern types
#[derive(Debug, Clone, Copy, PartialEq)]
enum PatternType {
    Checkerboard,
    Gradient,
    Text,
}

// Custom image view that forwards mouse events to our app delegate
define_class!(
    #[unsafe(super = NSImageView)]
    #[thread_kind = MainThreadOnly]
    #[name = "CustomImageView"]
    #[derive(Debug)]
    struct CustomImageView;

    unsafe impl NSObjectProtocol for CustomImageView {}

    impl CustomImageView {
        #[unsafe(method(mouseDown:))]
        fn mouseDown(&self, event: &NSEvent) {
            if let Some(delegate) = self.get_app_delegate() {
                unsafe {
                    let _: Bool = msg_send![delegate, mouseDown: event];
                }
            }

            unsafe {
                let _: () = msg_send![super(self), mouseDown: event];
            }
        }

        #[unsafe(method(mouseDragged:))]
        fn mouseDragged(&self, event: &NSEvent) {
            if let Some(delegate) = self.get_app_delegate() {
                unsafe {
                    let _: Bool = msg_send![delegate, mouseDragged: event];
                }
            }

            unsafe {
                let _: () = msg_send![super(self), mouseDragged: event];
            }
        }

        #[unsafe(method(mouseUp:))]
        fn mouseUp(&self, event: &NSEvent) {
            if let Some(delegate) = self.get_app_delegate() {
                unsafe {
                    let _: Bool = msg_send![delegate, mouseUp: event];
                }
            }

            unsafe {
                let _: () = msg_send![super(self), mouseUp: event];
            }
        }
    }
);

impl CustomImageView {
    fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        let this = Self::alloc(mtm);
        unsafe {
            let obj: Retained<Self> = msg_send![this, initWithFrame: frame];
            obj
        }
    }

    fn get_app_delegate(&self) -> Option<&AnyObject> {
        let mtm = self.mtm();
        let app = NSApplication::sharedApplication(mtm);

        unsafe {
            let delegate: *const AnyObject = msg_send![&*app, delegate];
            if delegate.is_null() {
                None
            } else {
                Some(&*delegate)
            }
        }
    }
}

// Add the CachedSourcePattern struct
#[derive(Debug)]
struct CachedSourcePattern {
    pattern: SourcePattern,
    pattern_type: PatternType,
    primary_text: Option<String>,
    secondary_text: Option<String>,
    source_width: usize,
    source_height: usize,
}

// Define the app delegate with ivars
#[derive(Debug, Default)]
struct AppDelegateIvars {
    window: OnceCell<Retained<NSWindow>>,
    scroll_view: OnceCell<Retained<NSScrollView>>,
    image_view: OnceCell<Retained<CustomImageView>>,
    selected_file_path: RefCell<Option<Retained<NSURL>>>,
    decoded_image: RefCell<Option<Retained<NSImage>>>,
    zoom_slider: OnceCell<Retained<NSSlider>>,
    last_mouse_location: RefCell<NSPoint>,
    is_panning: RefCell<bool>,
    magnification_recognizer: OnceCell<Retained<NSMagnificationGestureRecognizer>>,
    base_zoom_level: RefCell<f64>,
    state: RefCell<AppState>,
    cached_pattern: RefCell<Option<CachedSourcePattern>>,
}

// State container for state-forward architecture
#[derive(Debug, Clone)]
struct AppState {
    zoom_level: f64,
    pattern_type: PatternType,
    view_x: f64,
    view_y: f64,
    source_width: usize,
    source_height: usize,
    file_name: Option<String>,
    primary_text: Option<String>,
    secondary_text: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            pattern_type: PatternType::Text,
            view_x: 0.0,
            view_y: 0.0,
            source_width: 800,
            source_height: 600,
            file_name: None,
            primary_text: Some("COMING SOON".to_string()),
            secondary_text: None,
        }
    }
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[name = "AppDelegate"]
    #[ivars = AppDelegateIvars]
    struct AppDelegate;

    unsafe impl NSObjectProtocol for AppDelegate {}

    unsafe impl NSApplicationDelegate for AppDelegate {
        #[unsafe(method(applicationDidFinishLaunching:))]
        fn applicationDidFinishLaunching(&self, _notification: &NSNotification) {
            println!("DEBUG: Application did finish launching");

            let mtm = self.mtm();

            let window = self.create_window(mtm);
            let _ = self.ivars().window.set(window.clone());

            window.setTitle(ns_string!("JP2 Viewer"));
            window.center();

            self.setup_image_view(&window, mtm);
            self.setup_zoom_controls(&window, mtm);
            self.add_buttons(&window, mtm);
            self.setup_mouse_handling(&window);

            // Initialize default state
            {
                let mut state = self.ivars().state.borrow_mut();
                state.source_width = 800;
                state.source_height = 600;
                state.zoom_level = 1.0;
                state.pattern_type = PatternType::Text;
                state.primary_text = Some("COMING SOON".to_string());
            }

            // Initialize the pattern cache
            let _ = self.ensure_pattern_cache();

            // Render initial UI
            let _ = self.render_viewport();

            // Activate app and make window visible
            let app = NSApplication::sharedApplication(mtm);
            unsafe { app.activate() };
            window.makeKeyAndOrderFront(None);
        }
    }

    unsafe impl NSWindowDelegate for AppDelegate {
        #[unsafe(method(windowWillClose:))]
        fn windowWillClose(&self, _notification: &NSNotification) {
            let mtm = self.mtm();
            let app = NSApplication::sharedApplication(mtm);
            unsafe { app.terminate(None) };
        }
    }

    // Add custom methods for our delegate
    impl AppDelegate {
        #[unsafe(method(openFile:))]
        fn openFile(&self, _sender: Option<&NSObject>) -> Bool {
            println!("DEBUG: Opening file dialog");

            let mtm = self.mtm();
            let panel = unsafe { objc2_app_kit::NSOpenPanel::openPanel(mtm) };

            unsafe {
                panel.setCanChooseFiles(true);
                panel.setCanChooseDirectories(false);
                panel.setAllowsMultipleSelection(false);

                // Use UTType to specify JP2 content type
                let jp2_type = UTType::typeWithFilenameExtension(ns_string!("jp2"));
                if let Some(jp2_type) = jp2_type {
                    let allowed_types = NSArray::from_slice(&[&*jp2_type]);
                    panel.setAllowedContentTypes(&allowed_types);
                } else {
                    println!("DEBUG: Failed to create UTType for JP2, allowing all files");
                }

                let response = panel.runModal();

                if response == 1 {
                    let urls = panel.URLs();
                    if let Some(url) = urls.firstObject() {
                        println!("DEBUG: Selected file: {:?}", url);

                        // Keep a reference to the URL
                        *self.ivars().selected_file_path.borrow_mut() = Some(url.clone());

                        // Extract filename from URL
                        let filename = {
                            println!("DEBUG: Raw URL: {:?}", url);

                            let url_path = {
                                if let Some(path) = url.path().as_deref() {
                                    let ns_string = path.to_owned();
                                    format!("{}", &*ns_string)
                                } else {
                                    "unknown_path".to_string()
                                }
                            };

                            println!("DEBUG: Extracted path: {}", url_path);

                            url_path.split('/').last()
                                .unwrap_or("JP2 File")
                                .to_string()
                        };

                        println!("DEBUG: Showing Coming Soon text pattern for JP2 file: {:?}", &filename);

                        // Update state
                        {
                            let mut state = self.ivars().state.borrow_mut();
                            state.pattern_type = PatternType::Text;
                            state.primary_text = Some("COMING SOON".to_string());
                            state.secondary_text = Some(filename.clone());
                            state.file_name = Some(filename);

                            // Reset view position and zoom
                            state.view_x = 0.0;
                            state.view_y = 0.0;
                            state.zoom_level = 1.0;
                        }

                        // Full render (will regenerate pattern since content changed)
                        return self.render_ui();
                    }
                }
            }

            Bool::NO
        }

        #[unsafe(method(createGradient:))]
        fn createGradient(&self, _sender: Option<&NSObject>) -> Bool {
            println!("DEBUG: Creating gradient image");

            // Update state
            self.ivars().state.borrow_mut().pattern_type = PatternType::Gradient;

            // Full render (will regenerate pattern since type changed)
            self.render_ui()
        }

        #[unsafe(method(createCheckerboard:))]
        fn createCheckerboard(&self, _sender: Option<&NSObject>) -> Bool {
            println!("DEBUG: Creating checkerboard image");

            // Update state
            self.ivars().state.borrow_mut().pattern_type = PatternType::Checkerboard;

            // Full render (will regenerate pattern since type changed)
            self.render_ui()
        }

        #[unsafe(method(zoomChanged:))]
        fn zoomChanged(&self, sender: Option<&NSObject>) -> Bool {
            if let Some(obj) = sender {
                let slider_value: f64 = unsafe { msg_send![obj, doubleValue] };
                println!("DEBUG: Zoom changed to {}", slider_value);

                // Update state
                self.ivars().state.borrow_mut().zoom_level = slider_value.max(0.1).min(10.0);

                // Only render the viewport (not regenerate pattern)
                self.render_viewport()
            } else {
                Bool::NO
            }
        }

        #[unsafe(method(mouseDown:))]
        fn mouseDown(&self, event: &NSEvent) -> Bool {
            println!("DEBUG: Mouse down received");
            *self.ivars().is_panning.borrow_mut() = true;

            let location = unsafe { event.locationInWindow() };
            *self.ivars().last_mouse_location.borrow_mut() = location;

            Bool::YES
        }

        #[unsafe(method(mouseDragged:))]
        fn mouseDragged(&self, event: &NSEvent) -> Bool {
            println!("DEBUG: Mouse dragged");
            if *self.ivars().is_panning.borrow() {
                let current_location = unsafe { event.locationInWindow() };
                let last_location = *self.ivars().last_mouse_location.borrow();

                let delta_x = current_location.x - last_location.x;
                let delta_y = current_location.y - last_location.y;

                // Update state
                {
                    let mut state = self.ivars().state.borrow_mut();
                    state.view_x -= delta_x;
                    state.view_y -= delta_y;
                }

                // Only render the viewport (not regenerate pattern)
                let _ = self.render_viewport();

                *self.ivars().last_mouse_location.borrow_mut() = current_location;
                return Bool::YES;
            }

            Bool::NO
        }

        #[unsafe(method(mouseUp:))]
        fn mouseUp(&self, _event: &NSEvent) -> Bool {
            println!("DEBUG: Mouse up received");
            *self.ivars().is_panning.borrow_mut() = false;
            Bool::YES
        }

        #[unsafe(method(handlePinchGesture:))]
        fn handlePinchGesture(&self, sender: Option<&NSObject>) -> Bool {
            if let Some(recognizer) = sender {
                unsafe {
                    let state: isize = msg_send![recognizer, state];

                    // Handle different gesture states
                    if state == 1 { // GSBegan (1)
                        println!("DEBUG: Pinch gesture began");

                        // Store current zoom level as base for this gesture sequence
                        *self.ivars().base_zoom_level.borrow_mut() = self.ivars().state.borrow().zoom_level;
                    }

                    // Get the magnification factor from the gesture recognizer
                    let magnification: f64 = msg_send![recognizer, magnification];
                    println!("DEBUG: Pinch magnification: {}", magnification);

                    // Apply zoom change based on the base zoom level and magnification
                    let base_zoom = *self.ivars().base_zoom_level.borrow();
                    let new_zoom = base_zoom * (1.0 + magnification);

                    // Update state with new zoom level
                    self.ivars().state.borrow_mut().zoom_level = new_zoom.max(0.1).min(10.0);

                    // Only render the viewport (not regenerate pattern)
                    return self.render_viewport();
                }
            }

            Bool::NO
        }
    }
);

// Implement custom methods for AppDelegate
impl AppDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let ivars = AppDelegateIvars {
            base_zoom_level: RefCell::new(1.0),
            state: RefCell::new(AppState::default()),
            cached_pattern: RefCell::new(None),
            ..Default::default()
        };
        let this = Self::alloc(mtm).set_ivars(ivars);
        unsafe { msg_send![super(this), init] }
    }

    fn create_window(&self, mtm: MainThreadMarker) -> Retained<NSWindow> {
        let window_frame = NSRect::new(NSPoint::new(100., 100.), NSSize::new(800., 600.));
        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Resizable
            | NSWindowStyleMask::Miniaturizable;

        let window = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                window_frame,
                style,
                NSBackingStoreType::Buffered,
                false,
            )
        };

        // Important: prevent automatic closing from releasing the window
        // This is needed when not using a window controller
        unsafe { window.setReleasedWhenClosed(false) };

        window
    }

    fn setup_image_view(&self, window: &NSWindow, mtm: MainThreadMarker) {
        let content_view = window.contentView().unwrap();
        let content_frame = content_view.bounds();

        // Calculate the main view frame, leaving room for controls at the bottom
        let controls_height = 60.0;
        let main_view_frame = NSRect::new(
            NSPoint::new(0.0, controls_height),
            NSSize::new(
                content_frame.size.width,
                content_frame.size.height - controls_height,
            ),
        );

        // Create a scroll view
        let scroll_view =
            unsafe { NSScrollView::initWithFrame(NSScrollView::alloc(mtm), main_view_frame) };

        unsafe {
            scroll_view.setHasVerticalScroller(true);
            scroll_view.setHasHorizontalScroller(true);
            scroll_view.setAutoresizingMask(
                NSAutoresizingMaskOptions::ViewWidthSizable
                    | NSAutoresizingMaskOptions::ViewHeightSizable,
            );

            // Create our custom image view for the document view
            let frame = NSRect::ZERO;
            let new_image_view = CustomImageView::new(mtm, frame);

            // Configure image view properties
            new_image_view.setImageScaling(NSImageScaling::ScaleProportionallyDown);

            // Create and configure the magnification gesture recognizer for pinch-to-zoom
            let recognizer = NSMagnificationGestureRecognizer::alloc(mtm);
            let recognizer: Retained<NSMagnificationGestureRecognizer> =
                msg_send![recognizer, init];

            // Set the action and target for the gesture recognizer
            recognizer.setAction(Some(sel!(handlePinchGesture:)));
            let target: Option<&AnyObject> = Some(self.as_ref());
            recognizer.setTarget(target);

            // Add the gesture recognizer to the image view
            let view_ref: &AnyObject = new_image_view.as_ref();
            let _: () = msg_send![view_ref, addGestureRecognizer: &*recognizer];

            // Store the gesture recognizer
            let _ = self.ivars().magnification_recognizer.set(recognizer);

            // Set the image view as the document view
            scroll_view.setDocumentView(Some(&*new_image_view));

            // Add the scroll view to the content view
            content_view.addSubview(&scroll_view);

            // Store the views
            let _ = self.ivars().scroll_view.set(scroll_view.clone());
            let _ = self.ivars().image_view.set(new_image_view.clone());
        }
    }

    fn setup_zoom_controls(&self, window: &NSWindow, mtm: MainThreadMarker) {
        let content_view = window.contentView().unwrap();

        // Create a slider for zoom control
        let slider_frame = NSRect::new(NSPoint::new(530., 25.), NSSize::new(180., 30.));
        let slider = unsafe { NSSlider::initWithFrame(NSSlider::alloc(mtm), slider_frame) };

        unsafe {
            // Configure slider properties
            slider.setMinValue(0.1);
            slider.setMaxValue(5.0);
            slider.setDoubleValue(1.0);

            // Set number of tick marks directly using msg_send - use i64 (long) instead of i32
            let _: () = msg_send![&*slider, setNumberOfTickMarks: 9i64];
            let _: () = msg_send![&*slider, setAllowsTickMarkValuesOnly: false];

            // Set action and target
            slider.setAction(Some(sel!(zoomChanged:)));
            let target: Option<&AnyObject> = Some(self.as_ref());
            slider.setTarget(target);

            // Add to content view
            content_view.addSubview(&slider);

            // Store the slider
            let _ = self.ivars().zoom_slider.set(slider.clone());
        }
    }

    fn add_buttons(&self, window: &NSWindow, mtm: MainThreadMarker) {
        // Create Open JP2 button
        let open_button_frame = NSRect::new(NSPoint::new(20., 20.), NSSize::new(100., 30.));
        let open_button =
            unsafe { NSButton::initWithFrame(NSButton::alloc(mtm), open_button_frame) };

        unsafe {
            open_button.setTitle(ns_string!("Open JP2"));
            open_button.setBezelStyle(NSBezelStyle::Automatic);
            open_button.setAction(Some(sel!(openFile:)));

            // Convert self to AnyObject for target
            let target: Option<&AnyObject> = Some(self.as_ref());
            open_button.setTarget(target);

            let content_view = window.contentView().unwrap();
            content_view.addSubview(&open_button);
        }

        // Create Gradient button
        let gradient_button_frame = NSRect::new(NSPoint::new(140., 20.), NSSize::new(100., 30.));
        let gradient_button =
            unsafe { NSButton::initWithFrame(NSButton::alloc(mtm), gradient_button_frame) };

        unsafe {
            gradient_button.setTitle(ns_string!("Gradient"));
            gradient_button.setBezelStyle(NSBezelStyle::Automatic);
            gradient_button.setAction(Some(sel!(createGradient:)));

            // Convert self to AnyObject for target
            let target: Option<&AnyObject> = Some(self.as_ref());
            gradient_button.setTarget(target);

            let content_view = window.contentView().unwrap();
            content_view.addSubview(&gradient_button);
        }

        // Create Checkerboard button
        let checkerboard_button_frame =
            NSRect::new(NSPoint::new(260., 20.), NSSize::new(100., 30.));
        let checkerboard_button =
            unsafe { NSButton::initWithFrame(NSButton::alloc(mtm), checkerboard_button_frame) };

        unsafe {
            checkerboard_button.setTitle(ns_string!("Checkerboard"));
            checkerboard_button.setBezelStyle(NSBezelStyle::Automatic);
            checkerboard_button.setAction(Some(sel!(createCheckerboard:)));

            // Convert self to AnyObject for target
            let target: Option<&AnyObject> = Some(self.as_ref());
            checkerboard_button.setTarget(target);

            let content_view = window.contentView().unwrap();
            content_view.addSubview(&checkerboard_button);
        }
    }

    fn setup_mouse_handling(&self, _window: &NSWindow) {
        // Initial values
        *self.ivars().is_panning.borrow_mut() = false;
        *self.ivars().last_mouse_location.borrow_mut() = NSPoint::new(0.0, 0.0);

        // All mouse handling is now done through our CustomImageView subclass
        // that forwards events to our AppDelegate
        if let Some(window) = self.ivars().window.get() {
            window.setAcceptsMouseMovedEvents(true);
        }
    }

    // Central render function that updates UI based on state
    fn render_ui(&self) -> Bool {
        // First ensure we have the right pattern cached
        self.ensure_pattern_cache();

        // Then render the viewport based on current view parameters
        self.render_viewport()
    }

    // Ensure the pattern cache is up to date
    fn ensure_pattern_cache(&self) -> Bool {
        let state = self.ivars().state.borrow();
        let cache = self.ivars().cached_pattern.borrow();

        // Check if we need to regenerate the pattern
        let regenerate = match &*cache {
            None => true,
            Some(cached) => {
                cached.pattern_type != state.pattern_type
                    || cached.primary_text != state.primary_text
                    || cached.secondary_text != state.secondary_text
                    || cached.source_width != state.source_width
                    || cached.source_height != state.source_height
            }
        };

        if regenerate {
            // Generate new pattern and store in cache
            drop(cache); // Release the borrowed reference

            let source_pattern = self.generate_source_pattern_from_state(&*state);
            *self.ivars().cached_pattern.borrow_mut() = Some(CachedSourcePattern {
                pattern: source_pattern,
                pattern_type: state.pattern_type.clone(),
                primary_text: state.primary_text.clone(),
                secondary_text: state.secondary_text.clone(),
                source_width: state.source_width,
                source_height: state.source_height,
            });
        }

        Bool::YES
    }

    // Render the viewport based on current view parameters
    fn render_viewport(&self) -> Bool {
        let state = self.ivars().state.borrow();
        let cache = self.ivars().cached_pattern.borrow();

        if let Some(cached_pattern) = &*cache {
            // Create viewport image by transforming the cached source pattern
            if let Some(image) = self.generate_viewport_image(
                &cached_pattern.pattern,
                state.zoom_level,
                state.view_x,
                state.view_y,
            ) {
                // Store the generated image
                *self.ivars().decoded_image.borrow_mut() = Some(image.clone());

                // Update image view
                if let Some(image_view) = self.ivars().image_view.get() {
                    unsafe {
                        image_view.setImage(Some(&image));

                        let image_size = image.size();
                        let frame = NSRect::new(NSPoint::new(0.0, 0.0), image_size);
                        image_view.setFrame(frame);
                    }
                }

                // Update scroll view
                if let Some(scroll_view) = self.ivars().scroll_view.get() {
                    unsafe {
                        scroll_view
                            .documentView()
                            .unwrap()
                            .setFrame(self.ivars().image_view.get().unwrap().frame());
                        scroll_view.setNeedsDisplay(true);
                    }
                }

                // Update zoom slider to match current zoom level
                if let Some(slider) = self.ivars().zoom_slider.get() {
                    unsafe {
                        slider.setDoubleValue(state.zoom_level);
                    }
                }

                return Bool::YES;
            }
        }

        Bool::NO
    }

    // Generate viewport image from source pattern
    fn generate_viewport_image(
        &self,
        source_pattern: &SourcePattern,
        zoom_level: f64,
        view_x: f64,
        view_y: f64,
    ) -> Option<Retained<NSImage>> {
        // Viewport dimensions based on source dimensions and zoom level
        let viewport_width = (source_pattern.width as f64 * zoom_level) as usize;
        let viewport_height = (source_pattern.height as f64 * zoom_level) as usize;

        // Create a new image of the viewport size
        let size = NSSize::new(viewport_width as f64, viewport_height as f64);
        let alloc = NSImage::alloc();
        let image = unsafe { NSImage::initWithSize(alloc, size) };

        // Create a bitmap representation
        let alloc = NSBitmapImageRep::alloc();
        let color_space_name = ns_string!("NSDeviceRGBColorSpace");
        let bits_per_component = 8;
        let bytes_per_row = viewport_width * 4;

        let rep = unsafe {
            let planes: *const *mut u8 = std::ptr::null();
            let rep: Retained<NSBitmapImageRep> = msg_send![alloc,
                initWithBitmapDataPlanes: planes,
                pixelsWide: viewport_width as isize,
                pixelsHigh: viewport_height as isize,
                bitsPerSample: bits_per_component as isize,
                samplesPerPixel: 4 as isize,
                hasAlpha: true,
                isPlanar: false,
                colorSpaceName: &*color_space_name,
                bytesPerRow: bytes_per_row as isize,
                bitsPerPixel: 32 as isize
            ];

            rep
        };

        // Get bitmap data buffer
        let buffer: *mut u8 = unsafe { msg_send![&*rep, bitmapData] };

        if buffer.is_null() {
            println!("Failed to get bitmap data");
            return None;
        }

        // Apply zooming and panning to source pattern to generate final image
        unsafe {
            let scale_factor = 1.0 / zoom_level;
            let start_src_x = (view_x * scale_factor) as usize;
            let start_src_y = (view_y * scale_factor) as usize;

            for y in 0..viewport_height {
                for x in 0..viewport_width {
                    let dst_idx = (y * bytes_per_row + x * 4) as isize;

                    // Map viewport position to source coordinates
                    let src_x = start_src_x + (x as f64 * scale_factor) as usize;
                    let src_y = start_src_y + (y as f64 * scale_factor) as usize;

                    // Clamp to valid range
                    let src_x_clamped = src_x.min(source_pattern.width - 1);
                    let src_y_clamped = src_y.min(source_pattern.height - 1);

                    let src_idx = src_y_clamped * source_pattern.bytes_per_row + src_x_clamped * 4;

                    if src_idx + 3 < source_pattern.buffer.len() {
                        *buffer.offset(dst_idx) = source_pattern.buffer[src_idx];
                        *buffer.offset(dst_idx + 1) = source_pattern.buffer[src_idx + 1];
                        *buffer.offset(dst_idx + 2) = source_pattern.buffer[src_idx + 2];
                        *buffer.offset(dst_idx + 3) = source_pattern.buffer[src_idx + 3];
                    } else {
                        // Out of bounds - use purple
                        *buffer.offset(dst_idx) = 128;
                        *buffer.offset(dst_idx + 1) = 0;
                        *buffer.offset(dst_idx + 2) = 128;
                        *buffer.offset(dst_idx + 3) = 255;
                    }
                }
            }
        }

        // Add the bitmap representation to the image
        unsafe { image.addRepresentation(&rep) };

        Some(image)
    }

    // Generate source pattern based solely on state
    fn generate_source_pattern_from_state(&self, state: &AppState) -> SourcePattern {
        let width = state.source_width;
        let height = state.source_height;
        let bytes_per_row = width * 4;
        let buffer_size = bytes_per_row * height;
        let mut buffer = vec![0; buffer_size];

        match state.pattern_type {
            PatternType::Checkerboard => {
                self.generate_checkerboard_pattern(&mut buffer, width, height, bytes_per_row)
            }
            PatternType::Gradient => {
                self.generate_gradient_pattern(&mut buffer, width, height, bytes_per_row)
            }
            PatternType::Text => {
                self.generate_text_pattern(&mut buffer, width, height, bytes_per_row, state)
            }
        }

        self.add_debug_borders(&mut buffer, width, height, bytes_per_row);

        SourcePattern {
            buffer,
            width,
            height,
            bytes_per_row,
        }
    }

    // Generate a checkerboard pattern
    fn generate_checkerboard_pattern(
        &self,
        buffer: &mut Vec<u8>,
        width: usize,
        height: usize,
        bytes_per_row: usize,
    ) {
        let square_size = 20;

        for y in 0..height {
            for x in 0..width {
                let idx = y * bytes_per_row + x * 4;
                let is_white = ((x / square_size) + (y / square_size)) % 2 == 0;
                let color = if is_white { 255u8 } else { 0u8 };

                buffer[idx] = color;
                buffer[idx + 1] = color;
                buffer[idx + 2] = color;
                buffer[idx + 3] = 255;
            }
        }
    }

    // Generate a gradient pattern
    fn generate_gradient_pattern(
        &self,
        buffer: &mut Vec<u8>,
        width: usize,
        height: usize,
        bytes_per_row: usize,
    ) {
        for y in 0..height {
            for x in 0..width {
                let idx = y * bytes_per_row + x * 4;
                let r = ((x as f64) / (width as f64) * 255.0) as u8;
                let g = ((y as f64) / (height as f64) * 255.0) as u8;
                let b = 200u8;

                buffer[idx] = r;
                buffer[idx + 1] = g;
                buffer[idx + 2] = b;
                buffer[idx + 3] = 255;
            }
        }
    }

    // Generate a text pattern
    fn generate_text_pattern(
        &self,
        buffer: &mut Vec<u8>,
        width: usize,
        height: usize,
        bytes_per_row: usize,
        state: &AppState,
    ) {
        // Fill with light blue-gray background
        for y in 0..height {
            for x in 0..width {
                let idx = y * bytes_per_row + x * 4;
                buffer[idx] = 230;
                buffer[idx + 1] = 235;
                buffer[idx + 2] = 240;
                buffer[idx + 3] = 255;
            }
        }

        let char_map: std::collections::HashMap<char, usize> =
            CHAR_INDICES.iter().cloned().collect();

        let primary = state.primary_text.as_deref().unwrap_or("COMING SOON");

        // Text sizing and positioning
        let char_width = 32;
        let char_height = 40;
        let char_padding = 4;

        let text_width = primary.len() * (char_width + char_padding);
        let start_x = (width - text_width) / 2;
        let start_y = height / 2 - char_height;

        // Draw primary text
        self.draw_text(
            buffer,
            width,
            height,
            bytes_per_row,
            &BITMAP_CHARS,
            &char_map,
            primary,
            start_x,
            start_y,
            char_width,
            char_height,
            char_padding,
            [30, 30, 180], // Dark blue
        );

        // Draw secondary text if available
        if let Some(secondary) = &state.secondary_text {
            let secondary_text = secondary;
            let smaller_char_width = 16;
            let smaller_char_height = 20;
            let smaller_padding = 2;

            // Limit text length if needed
            let display_text = if secondary_text.len() > 30 {
                format!("{}...", &secondary_text[0..27])
            } else {
                secondary_text.to_string()
            };

            let secondary_text_width = display_text.len() * (smaller_char_width + smaller_padding);
            let secondary_x = (width - secondary_text_width) / 2;
            let secondary_y = start_y + char_height + 40; // Below primary text

            self.draw_text(
                buffer,
                width,
                height,
                bytes_per_row,
                &BITMAP_CHARS,
                &char_map,
                &display_text.to_uppercase(),
                secondary_x,
                secondary_y,
                smaller_char_width,
                smaller_char_height,
                smaller_padding,
                [20, 120, 20], // Dark green
            );
        }

        // Add "FILE SELECTED" text if there's a secondary text
        if state.secondary_text.is_some() {
            let info_text = "FILE SELECTED";
            let small_char_width = 12;
            let small_char_height = 15;
            let small_padding = 1;

            let info_text_width = info_text.len() * (small_char_width + small_padding);
            let info_x = (width - info_text_width) / 2;
            let info_y = height - 60; // Near bottom

            self.draw_text(
                buffer,
                width,
                height,
                bytes_per_row,
                &BITMAP_CHARS,
                &char_map,
                info_text,
                info_x,
                info_y,
                small_char_width,
                small_char_height,
                small_padding,
                [150, 50, 50], // Red
            );
        }
    }

    // Helper to draw text with the bitmap font
    fn draw_text(
        &self,
        buffer: &mut Vec<u8>,
        width: usize,
        height: usize,
        bytes_per_row: usize,
        characters: &[[[u8; 5]; 5]],
        char_map: &std::collections::HashMap<char, usize>,
        text: &str,
        start_x: usize,
        start_y: usize,
        char_width: usize,
        char_height: usize,
        char_padding: usize,
        color: [u8; 3],
    ) {
        // Scale factors to expand the 5x5 bitmap
        let scale_x = char_width / 5;
        let scale_y = char_height / 5;

        for (i, c) in text.chars().enumerate() {
            let char_idx = char_map.get(&c).copied().unwrap_or(10); // Default to space
            let bitmap = &characters[char_idx];
            let char_x = start_x + i * (char_width + char_padding);

            for (y_idx, row) in bitmap.iter().enumerate() {
                for (x_idx, &pixel) in row.iter().enumerate() {
                    if pixel == 1 {
                        for sy in 0..scale_y {
                            for sx in 0..scale_x {
                                let x = char_x + x_idx * scale_x + sx;
                                let y = start_y + y_idx * scale_y + sy;

                                if x >= width || y >= height {
                                    continue;
                                }

                                let idx = y * bytes_per_row + x * 4;
                                if idx + 3 < buffer.len() {
                                    buffer[idx] = color[0];
                                    buffer[idx + 1] = color[1];
                                    buffer[idx + 2] = color[2];
                                    buffer[idx + 3] = 255;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Add debug borders and corner markers to the source pattern
    fn add_debug_borders(
        &self,
        buffer: &mut Vec<u8>,
        width: usize,
        height: usize,
        bytes_per_row: usize,
    ) {
        let border_thickness = 3;
        let corner_size = 15;

        // Color definitions for borders and corner markers
        let red = [255u8, 0, 0, 255];
        let green = [0u8, 255, 0, 255];
        let blue = [0u8, 0, 255, 255];
        let yellow = [255u8, 255, 0, 255];

        // Draw top and bottom borders
        for y in 0..border_thickness {
            // Top edge
            for x in 0..width {
                let idx = y * bytes_per_row + x * 4;
                buffer[idx] = red[0];
                buffer[idx + 1] = red[1];
                buffer[idx + 2] = red[2];
                buffer[idx + 3] = red[3];
            }

            // Bottom edge
            if height > border_thickness {
                for x in 0..width {
                    let idx = (height - 1 - y) * bytes_per_row + x * 4;
                    buffer[idx] = red[0];
                    buffer[idx + 1] = red[1];
                    buffer[idx + 2] = red[2];
                    buffer[idx + 3] = red[3];
                }
            }
        }

        // Draw left and right borders
        for x in 0..border_thickness {
            // Left edge
            for y in 0..height {
                let idx = y * bytes_per_row + x * 4;
                buffer[idx] = red[0];
                buffer[idx + 1] = red[1];
                buffer[idx + 2] = red[2];
                buffer[idx + 3] = red[3];
            }

            // Right edge
            if width > border_thickness {
                for y in 0..height {
                    let idx = y * bytes_per_row + (width - 1 - x) * 4;
                    buffer[idx] = red[0];
                    buffer[idx + 1] = red[1];
                    buffer[idx + 2] = red[2];
                    buffer[idx + 3] = red[3];
                }
            }
        }

        // Draw colored corner boxes
        self.draw_corner_box(buffer, bytes_per_row, 0, 0, corner_size, red);

        if width > corner_size {
            self.draw_corner_box(
                buffer,
                bytes_per_row,
                width - corner_size,
                0,
                corner_size,
                green,
            );
        }

        if height > corner_size {
            self.draw_corner_box(
                buffer,
                bytes_per_row,
                0,
                height - corner_size,
                corner_size,
                blue,
            );
        }

        if width > corner_size && height > corner_size {
            self.draw_corner_box(
                buffer,
                bytes_per_row,
                width - corner_size,
                height - corner_size,
                corner_size,
                yellow,
            );
        }
    }

    fn draw_corner_box(
        &self,
        buffer: &mut Vec<u8>,
        bytes_per_row: usize,
        start_x: usize,
        start_y: usize,
        size: usize,
        color: [u8; 4],
    ) {
        for y in 0..size {
            for x in 0..size {
                let idx = (start_y + y) * bytes_per_row + (start_x + x) * 4;
                if idx + 3 < buffer.len() {
                    buffer[idx] = color[0];
                    buffer[idx + 1] = color[1];
                    buffer[idx + 2] = color[2];
                    buffer[idx + 3] = color[3];
                }
            }
        }
    }
}

fn main() {
    // Initialize on the main thread
    let mtm = MainThreadMarker::new().expect("Not running on main thread");

    // Get the shared application instance
    let app = NSApplication::sharedApplication(mtm);

    // Set the activation policy
    app.setActivationPolicy(NSApplicationActivationPolicy::Regular);

    // Create our app delegate
    let delegate = AppDelegate::new(mtm);

    // Set the delegate
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    // Activation is now done in applicationDidFinishLaunching
    // to properly sequence window visibility

    println!("DEBUG: Starting application run loop");
    app.run();
}
