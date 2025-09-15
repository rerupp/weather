//! The UI basic controls.
//!
//! This module provides some basic UI controls. The controls include:
//!
//! - a [button](Button) that can invoke some action.
//! - a [button bar](ButtonBar) manages a collection of buttons.
//! - a [checkbox](Checkbox) that can be set active or not.
//! - a [checkbox group](CheckBoxGroup) manages a collection of checkbox controls.
//! - a [label](Label) providing a readonly text description.
//! - a [report viewer](ReportView) renders the contents of a [toolslib report sheet](toolslib::report::ReportSheet).
//! - a [date field](DateEditor) editor.
//! - a [text field](TextEditor) editor.
//! - a [group](EditFieldGroup) manages a collection of date and text editors.
//!
//! In general all controls implement the [Control] trait.
//!
use super::*;
pub use button::{cancel_button, ok_button, Button, ButtonBar, CANCEL_BUTTON_ID, OK_BUTTON_ID};
pub use checkbox::{Checkbox, CheckBoxGroup};
pub use label::Label;
pub use report_view::ReportView;
use std::ops::ControlFlow;
use styles::{ActiveNormalStyles, StyleCatalog};
pub use text::{DateEditor, EditField, EditFieldGroup, Editor, TextEditor};

mod button;
mod checkbox;
mod label;
mod report_view;
mod text;

/// The common control API.
pub trait Control {
    /// Get the control identifier.
    ///
    fn id(&self) -> &str;
    /// Get the control selector character.
    ///
    fn selector(&self) -> char;
    /// Get the size of the control
    ///
    fn size(&self) -> Size;
    /// Indicates the control is active.
    ///
    fn is_active(&self) -> bool;
    /// Set the control active or not.
    ///
    fn set_active(&mut self, active: bool);
    /// Draw the control on the terminal screen and return the coordinate of the controls selector if active.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the button will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](StyleCatalog) used the draw the button.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, styles: &StyleCatalog) -> Option<Position>;
    /// Pass a key pressed event to the control, it should return [ControlFlow::Continue]
    /// if the event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key press](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult>;
}

/// The API used for [date](DateEditor) and [text][TextEditor] field editors.
pub trait EditControl: Control {
    /// Get the raw text in the editor.
    ///
    fn text(&self) -> &str;
    /// Set the text in the editor.
    ///
    /// # Arguments
    ///
    /// - `text` is the text that will be set into the editor.
    ///
    fn set_text(&mut self, text: impl ToString);
}

/// The API used for a collection of like controls.
pub trait ControlGroup<T: Control> {
    /// Get the size of the control group.
    ///
    fn size(&self) -> Size;
    /// Get a control from the group.
    ///
    /// # Arguments
    ///
    /// - `id` is the control identifier.
    ///
    fn get(&self, id: impl ToString) -> Option<&T>;
    /// Get a mutable control from the group.
    ///
    /// # Arguments
    ///
    /// - `id` is the control identifier.
    ///
    fn get_mut(&mut self, id: impl ToString) -> Option<&mut T>;
    /// Set a control within the group active.
    ///
    /// # Arguments
    ///
    /// - `id` is the control identifier.
    ///
    fn set_active(&mut self, id: impl ToString) -> ControlFlow<ControlResult>;
    /// Dispatch a key pressed event to the group. [ControlFlow::Continue] should be returned if the
    /// event is not consumed.
    ///
    /// # Arguments
    ///
    /// - `key_event` is guaranteed to be a [key pressed](KeyEventKind::Press) event.
    ///
    fn key_pressed(&mut self, key_event: KeyEvent) -> ControlFlow<ControlResult>;
    /// Get a reference to the controls in the group.
    ///
    fn controls(&self) -> Vec<&T>;
    /// Draw the control group on the terminal screen and return the coordinate of the active control selector.
    ///
    /// # Arguments
    ///
    /// - `area` is where on the terminal the control group will be drawn.
    /// - `buffer` is the current view of the terminal screen.
    /// - `styles` contains the [styles](ActiveNormalStyles) used the draw the control group.
    ///
    fn render(&self, area: Rect, buffer: &mut Buffer, styles: ActiveNormalStyles) -> Option<Position>;
    /// Resets all controls to a not active state.
    ///
    fn clear_active(&mut self);
    /// Set the first control in the group to an active state
    ///
    fn set_first_active(&mut self);
    /// Set the last control in the group to an active state
    ///
    fn set_last_active(&mut self);
}

/// The result of a control event.
#[derive(Debug, PartialOrd, PartialEq)]
pub enum ControlResult {
    /// Indicate the event is cancelled.
    Cancel,
    /// Indicate the event has consumed.
    Continue,
    /// Returns some type of error condition.
    Error(String),
    /// Indicate the next group of controls should be used,
    NextGroup,
    /// Indicate the event has consumed however it was not allowed.
    NotAllowed,
    /// Indicate the previous group of controls should be used.
    PrevGroup,
    /// Returns the identifier of some control.
    Selected(String),
}

/// Used by controls to create styled text hot hotkey fields like button, menu, etc.
///
/// Arguments
///
/// * `text` is the string hotkey spans will be created from.
/// * `hotkey` is the hotkey character.
/// * `text_style` is the strings style.
/// * `hotkey_style` is the hotkey character style.
///
pub fn hotkey_spans(text: &str, hotkey: char, text_style: Style, hotkey_style: Style) -> Vec<Span> {
    match text.split_once(hotkey) {
        Some((lhs, rhs)) => {
            let mut spans: Vec<Span> = Vec::with_capacity(3);
            if lhs.len() > 0 {
                spans.push(Span::styled(lhs, text_style))
            }
            spans.push(Span::styled(hotkey.to_string(), hotkey_style));
            if rhs.len() > 0 {
                spans.push(Span::styled(rhs, text_style))
            }
            spans
        }
        None => vec![Span::styled(text, text_style)],
    }
}

/// For a collection of controls, find the currently active control and set the next control active.
///
/// # Arguments
///
/// - `controls` is the collection that will be updated
/// - `wrap` is used to set the first control in the collection active if the last control is currently active.
///
fn next_control(mut controls: Vec<&mut impl Control>, wrap: bool) -> bool {
    let controls_len = controls.len();
    match controls_len < 2 {
        true => false,
        false => match controls.last().unwrap().is_active() {
            true => {
                if wrap {
                    controls.first_mut().unwrap().set_active(true);
                    controls.last_mut().unwrap().set_active(false);
                }
                wrap
            }
            false => {
                for idx in 0..controls_len - 1 {
                    if controls[idx].is_active() {
                        controls[idx].set_active(false);
                        controls[idx + 1].set_active(true);
                        break;
                    }
                }
                // assumes there is always an active control
                true
            }
        },
    }
}

/// For a collection of controls, find the currently active control and set the previous control active.
///
/// # Arguments
///
/// - `controls` is the collection that will be updated.
/// - `wrap` is used to set the last control in the collection active if the first control is currently active.
///
fn previous_control(mut controls: Vec<&mut impl Control>, wrap: bool) -> bool {
    let controls_len = controls.len();
    match controls_len < 2 {
        true => false,
        false => match controls.first().unwrap().is_active() {
            true => {
                if wrap {
                    controls.first_mut().unwrap().set_active(false);
                    controls.last_mut().unwrap().set_active(true);
                }
                wrap
            }
            false => {
                for idx in 1..controls_len {
                    if controls[idx].is_active() {
                        controls[idx].set_active(false);
                        controls[idx - 1].set_active(true);
                        break;
                    }
                }
                // assumes there is always an active control
                true
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestControl(bool);
    #[allow(unused)]
    impl Control for TestControl {
        fn id(&self) -> &str {
            todo!()
        }
        fn selector(&self) -> char {
            todo!()
        }
        fn size(&self) -> Size {
            todo!()
        }
        fn is_active(&self) -> bool {
            self.0
        }
        fn set_active(&mut self, active: bool) {
            self.0 = active;
        }
        fn render(&self, _: Rect, _: &mut Buffer, _: &StyleCatalog) -> Option<Position> {
            None
        }
        fn key_pressed(&mut self, key_event: &KeyEvent) -> ControlFlow<ControlResult> {
            ControlFlow::Continue(())
        }
    }

    #[test]
    fn next_test() {
        // verify having less < 2 controls
        let mut testcase = vec![TestControl(false)];
        // assert!(!next_control(testcase.iter_mut().collect(), false));
        assert!(!next_control(as_mut_refs!(testcase), false));
        assert!(!next_control(as_mut_refs!(testcase), true));
        testcase[0].0 = true;
        assert!(!next_control(as_mut_refs!(testcase), false));
        assert!(!next_control(as_mut_refs!(testcase), true));
        // setup a wrapping scenario
        let mut testcase = vec![TestControl(false), TestControl(false), TestControl(true)];
        assert!(!next_control(as_mut_refs!(testcase), false));
        assert!(!testcase[0].is_active());
        assert!(!testcase[1].is_active());
        assert!(testcase[2].is_active());
        assert!(next_control(as_mut_refs!(testcase), true));
        assert!(testcase[0].is_active());
        assert!(!testcase[1].is_active());
        assert!(!testcase[2].is_active());
        assert!(next_control(as_mut_refs!(testcase), false));
        assert!(!testcase[0].is_active());
        assert!(testcase[1].is_active());
        assert!(!testcase[2].is_active());
    }

    #[test]
    fn prev_test() {
        // verify having less < 2 controls
        let mut testcase = vec![TestControl(false)];
        assert!(!previous_control(as_mut_refs!(testcase), false));
        assert!(!previous_control(as_mut_refs!(testcase), true));
        testcase[0].0 = true;
        assert!(!previous_control(as_mut_refs!(testcase), false));
        assert!(!previous_control(as_mut_refs!(testcase), true));
        // setup a wrapping scenario
        let mut testcase = vec![TestControl(true), TestControl(false), TestControl(false)];
        assert!(!previous_control(as_mut_refs!(testcase), false));
        assert!(testcase[0].is_active());
        assert!(!testcase[1].is_active());
        assert!(!testcase[2].is_active());
        assert!(previous_control(as_mut_refs!(testcase), true));
        assert!(!testcase[0].is_active());
        assert!(!testcase[1].is_active());
        assert!(testcase[2].is_active());
        assert!(previous_control(as_mut_refs!(testcase), false));
        assert!(!testcase[0].is_active());
        assert!(testcase[1].is_active());
        assert!(!testcase[2].is_active());
    }

    #[test]
    fn areas() {
        // rows 0..9, column 0..9
        let area = Rect { x: 0, y: 0, width: 10, height: 10 };
        let testcase = inner_rect(area, (0, 0), (-1, -1));
        assert_eq!(testcase, Rect { x: 0, y: 0, width: 9, height: 9 });
        let testcase = inner_rect(area, (0, 0), (9, 9));
        assert_eq!(testcase, Rect { x: 0, y: 0, width: 9, height: 9 });
        let testcase = inner_rect(area, (-9, -9), (0, 0));
        assert_eq!(testcase, Rect { x: 1, y: 1, width: 9, height: 9 });
        let testcase = inner_rect(area, (0, 0), (-9, -9));
        assert_eq!(testcase, Rect { x: 0, y: 0, width: 1, height: 1 });
        let testcase = inner_rect(area, (0, 0), (0, 0));
        assert_eq!(testcase, Rect { x: 0, y: 0, width: 10, height: 10 });
        let testcase = inner_rect(area, (1, 1), (0, 0));
        assert_eq!(testcase, Rect { x: 1, y: 1, width: 9, height: 9 });
        let testcase = inner_rect(area, (0, 0), (-1, -1));
        assert_eq!(testcase, Rect { x: 0, y: 0, width: 9, height: 9 });
        let testcase = inner_rect(area, (1, 1), (-1, -1));
        assert_eq!(testcase, Rect { x: 1, y: 1, width: 8, height: 8 });
        let testcase = inner_rect(area, (2, 2), (8, 8));
        assert_eq!(testcase, Rect { x: 2, y: 2, width: 6, height: 6 });
        let testcase = inner_rect(area, (2, 2), (-2, -2));
        assert_eq!(testcase, Rect { x: 2, y: 2, width: 6, height: 6 });
        let testcase = inner_rect(area, (6, 6), (5, 5));
        assert_eq!(testcase, Rect { x: 6, y: 6, width: 0, height: 0 });
        let testcase = inner_rect(area, (11, 11), (0, 0));
        assert_eq!(testcase, Rect { x: 10, y: 10, width: 0, height: 0 });
        let testcase = inner_rect(area, (0, 0), (11, 11));
        assert_eq!(testcase, Rect { x: 0, y: 0, width: 10, height: 10 });
        let testcase = inner_rect(area, (0, 0), (-10, -10));
        assert_eq!(testcase, Rect { x: 0, y: 0, width: 0, height: 0 });
        let testcase = inner_rect(area, (-10, -10), (0, 0));
        assert_eq!(testcase, Rect { x: 0, y: 0, width: 10, height: 10 });
        let testcase = inner_rect(area, (0, -5), (5, 0));
        assert_eq!(testcase, Rect { x: 0, y: 5, width: 5, height: 5 });
    }
}
