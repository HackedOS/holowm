use std::{cell::RefCell, rc::Rc};

use smithay::{
    desktop::Window,
    utils::{Logical, Physical, Point, Rectangle, Size},
};

use super::{
    binarytree::{BinaryTree, HorizontalOrVertical},
    workspaces::{HoloWindow, Workspace},
};

pub enum WindowLayoutEvent {
    Added,
    Removed,
    Resized,
}

pub fn bsp_layout(
    workspace: &mut Workspace,
    window: Window,
    event: WindowLayoutEvent,
    gaps: (i32, i32),
) {
    let output = workspace
        .outputs()
        .next()
        .unwrap()
        .current_mode()
        .unwrap()
        .size;

    match event {
        WindowLayoutEvent::Added => {
            let window = Rc::new(RefCell::new(HoloWindow {
                window,
                rec: Rectangle {
                    loc: Point::from((gaps.0, gaps.0)),
                    size: Size::from((output.w - (gaps.0 * 2), output.h - (gaps.0 * 2))),
                },
            }));
            workspace.add_window(window);

            bsp_update_layout(workspace, gaps);
        }
        WindowLayoutEvent::Removed => {
            workspace.remove_window(&window);
            bsp_update_layout(workspace, gaps);
        }
        WindowLayoutEvent::Resized => todo!(),
    }
    dbg!(workspace.layout_tree.clone());
}

pub fn bsp_update_layout(workspace: &mut Workspace, gaps: (i32, i32)) {
    //recalculate the size and location of the windows

    let output = workspace
        .outputs()
        .next()
        .unwrap()
        .current_mode()
        .unwrap()
        .size;

    match &mut workspace.layout_tree {
        BinaryTree::Empty => {}
        BinaryTree::Window(w) => {
            w.borrow_mut().rec = Rectangle {
                loc: Point::from((gaps.0 + gaps.1, gaps.0 + gaps.1)),
                size: Size::from((
                    output.w - ((gaps.0 + gaps.1) * 2),
                    output.h - ((gaps.0 + gaps.1) * 2),
                )),
            };
        }
        BinaryTree::Split {
            left,
            right,
            split,
            ratio,
        } => {
            if let BinaryTree::Window(w) = left.as_mut() {
                generate_layout(
                    right.as_mut(),
                    &w,
                    Rectangle {
                        loc: Point::from((gaps.0, gaps.0)),
                        size: Size::from((output.w - (gaps.0 * 2), output.h - (gaps.0 * 2))),
                    },
                    *split,
                    *ratio,
                    Size::from((output.w - (gaps.0), output.h - (gaps.0))),
                    gaps,
                )
            }
        }
    }
    for holowindow in workspace.holowindows() {
        let xdg_toplevel = holowindow.window.toplevel();
        xdg_toplevel.with_pending_state(|state| {
            state.size = Some(holowindow.rec.size);
        });
        xdg_toplevel.send_configure();
    }
}

pub fn generate_layout(
    tree: &mut BinaryTree,
    lastwin: &Rc<RefCell<HoloWindow>>,
    lastgeo: Rectangle<i32, Logical>,
    split: HorizontalOrVertical,
    ratio: f32,
    output: Size<i32, Physical>,
    gaps: (i32, i32),
) {
    let size;
    match split {
        HorizontalOrVertical::Horizontal => {
            size = Size::from((lastgeo.size.w / 2, lastgeo.size.h));
        }
        HorizontalOrVertical::Vertical => {
            size = Size::from((lastgeo.size.w, lastgeo.size.h / 2));
        }
    }

    let loc: Point<i32, Logical>;
    match split {
        HorizontalOrVertical::Horizontal => {
            loc = Point::from((lastgeo.loc.x, output.h - size.h));
        }
        HorizontalOrVertical::Vertical => {
            loc = Point::from((output.w - size.w, lastgeo.loc.y));
        }
    }

    let recgapped = Rectangle {
        size: Size::from((size.w - (gaps.1 * 2), (size.h - (gaps.1 * 2)))),
        loc: Point::from((loc.x + gaps.1, loc.y + gaps.1)),
    };

    lastwin.borrow_mut().rec = recgapped;

    let loc;
    match split {
        HorizontalOrVertical::Horizontal => {
            loc = Point::from((output.w - size.w, lastgeo.loc.y));
        }
        HorizontalOrVertical::Vertical => {
            loc = Point::from((lastgeo.loc.x, output.h - size.h));
        }
    }

    let rec = Rectangle { size, loc };
    let recgapped = Rectangle {
        size: Size::from((size.w - (gaps.1 * 2), (size.h - (gaps.1 * 2)))),
        loc: Point::from((loc.x + gaps.1, loc.y + gaps.1)),
    };
    match tree {
        BinaryTree::Empty => {}
        BinaryTree::Window(w) => w.borrow_mut().rec = recgapped,
        BinaryTree::Split {
            split,
            ratio,
            left,
            right,
        } => {
            if let BinaryTree::Window(w) = left.as_mut() {
                w.borrow_mut().rec = rec;
                generate_layout(right.as_mut(), &w, rec, *split, *ratio, output, gaps)
            }
        }
    }
}
