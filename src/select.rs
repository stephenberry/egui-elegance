//! Styled select (combo-box) widget.
//!
//! Wraps [`egui::ComboBox`] and paints it with the elegance palette: slate
//! input background, 1-px border, sky focus ring, and a matching chevron.

use std::borrow::Cow;
use std::hash::Hash;

use egui::{Color32, ComboBox, Response, Stroke, Ui, Widget, WidgetInfo, WidgetText, WidgetType};

use crate::theme::Theme;

/// A styled drop-down select.
///
/// Bind the selection to any `PartialEq + Clone` type — an enum, an index,
/// or a `String` — and supply a list of `(value, label)` pairs. Labels
/// accept `&'static str`, `String`, or any `Cow<'a, str>`, so static option
/// lists don't allocate.
///
/// ```no_run
/// # use elegance::Select;
/// # egui::__run_test_ui(|ui| {
/// #[derive(Clone, PartialEq)]
/// enum Unit { Us, Ms, S }
///
/// let mut unit = Unit::Ms;
/// ui.add(Select::new("unit", &mut unit).options([
///     (Unit::Us, "μs"),
///     (Unit::Ms, "ms"),
///     (Unit::S,  "s"),
/// ]));
/// # });
/// ```
///
/// For string-valued selects where each option is both the value and the
/// label, use [`Select::strings`]:
///
/// ```no_run
/// # use elegance::Select;
/// # egui::__run_test_ui(|ui| {
/// let mut unit = String::from("ms");
/// ui.add(Select::strings("unit", &mut unit, ["us", "ms", "s"]));
/// # });
/// ```
#[must_use = "Add with `ui.add(...)`."]
pub struct Select<'a, T: PartialEq + Clone> {
    id_salt: egui::Id,
    value: &'a mut T,
    label: Option<WidgetText>,
    options: Vec<(T, Cow<'a, str>)>,
    width: Option<f32>,
}

impl<'a, T: PartialEq + Clone> std::fmt::Debug for Select<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let labels: Vec<&str> = self.options.iter().map(|(_, l)| l.as_ref()).collect();
        f.debug_struct("Select")
            .field("id_salt", &self.id_salt)
            .field("option_labels", &labels)
            .field("width", &self.width)
            .finish()
    }
}

impl<'a, T: PartialEq + Clone> Select<'a, T> {
    /// Create a select keyed by `id_salt` and bound to `value`.
    /// Add selectable options via [`Select::options`].
    pub fn new(id_salt: impl Hash, value: &'a mut T) -> Self {
        Self {
            id_salt: egui::Id::new(id_salt),
            value,
            label: None,
            options: Vec::new(),
            width: None,
        }
    }

    /// Show a label above the select.
    pub fn label(mut self, label: impl Into<WidgetText>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the selectable options as `(value, label)` pairs. Labels are
    /// carried as `Cow<'a, str>`, so `&'static str` labels never allocate.
    pub fn options<I, S>(mut self, options: I) -> Self
    where
        I: IntoIterator<Item = (T, S)>,
        S: Into<Cow<'a, str>>,
    {
        self.options = options.into_iter().map(|(v, l)| (v, l.into())).collect();
        self
    }

    /// Override the select width in points. Defaults to the intrinsic
    /// size of the selected label plus padding.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
}

impl<'a> Select<'a, String> {
    /// Convenience constructor for string-valued selects. Each item is used
    /// as both the value and the displayed label.
    ///
    /// ```no_run
    /// # use elegance::Select;
    /// # egui::__run_test_ui(|ui| {
    /// let mut unit = String::from("ms");
    /// ui.add(Select::strings("unit", &mut unit, ["us", "ms", "s"]));
    /// # });
    /// ```
    pub fn strings<I, S>(id_salt: impl Hash, value: &'a mut String, options: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Cow<'a, str>>,
    {
        let options: Vec<(String, Cow<'a, str>)> = options
            .into_iter()
            .map(|s| {
                let label: Cow<'a, str> = s.into();
                let value = label.as_ref().to_owned();
                (value, label)
            })
            .collect();
        Self {
            id_salt: egui::Id::new(id_salt),
            value,
            label: None,
            options,
            width: None,
        }
    }
}

impl<'a, T: PartialEq + Clone> Widget for Select<'a, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = Theme::current(ui.ctx());
        let p = &theme.palette;
        let t = &theme.typography;

        ui.vertical(|ui| {
            if let Some(label) = &self.label {
                let rich = egui::RichText::new(label.text())
                    .color(p.text_muted)
                    .size(t.label);
                ui.add(egui::Label::new(rich).wrap_mode(egui::TextWrapMode::Extend));
                ui.add_space(2.0);
            }

            let width = self.width.unwrap_or(160.0);
            let chevron_color = p.text_muted;

            // Resolve the displayed label for the current value. Owned so
            // it doesn't conflict with the mutable access to `self.value`
            // in the inner closure.
            let selected_label: String = self
                .options
                .iter()
                .find(|(v, _)| v == &*self.value)
                .map(|(_, l)| l.as_ref().to_owned())
                .unwrap_or_default();
            let field_label = self.label.as_ref().map(|l| l.text().to_string());

            let response = crate::theme::with_themed_visuals(ui, |ui| {
                let v = ui.visuals_mut();
                crate::theme::themed_input_visuals(v, &theme, p.input_bg);
                for w in [
                    &mut v.widgets.inactive,
                    &mut v.widgets.hovered,
                    &mut v.widgets.active,
                    &mut v.widgets.open,
                ] {
                    w.fg_stroke = Stroke::new(1.0, p.text);
                }
                v.override_text_color = Some(p.text);

                ComboBox::from_id_salt(self.id_salt)
                    .width(width)
                    .selected_text(
                        egui::RichText::new(&selected_label)
                            .color(p.text)
                            .size(t.body),
                    )
                    .icon(move |ui, rect, _visuals, is_popup_open| {
                        paint_chevron(ui, rect, chevron_color, is_popup_open);
                    })
                    .show_ui(ui, |ui| {
                        ui.set_min_width(width);
                        for (opt_value, opt_label) in self.options.iter() {
                            let label = egui::RichText::new(opt_label.as_ref())
                                .color(p.text)
                                .size(t.body);
                            if ui
                                .selectable_label(opt_value == &*self.value, label)
                                .clicked()
                            {
                                *self.value = opt_value.clone();
                            }
                        }
                    })
                    .response
            });

            if let Some(field_label) = field_label {
                let selected_label = selected_label.clone();
                response.widget_info(|| {
                    let mut info = WidgetInfo::labeled(WidgetType::ComboBox, true, &field_label);
                    info.current_text_value = Some(selected_label.clone());
                    info
                });
            }

            response
        })
        .inner
    }
}

/// Paint a thin, centered chevron inside `rect`. Points down when the popup is
/// closed (hint to open) and flips up when the popup is open (hint to close).
fn paint_chevron(ui: &egui::Ui, rect: egui::Rect, color: Color32, is_popup_open: bool) {
    let painter = ui.painter();
    let stroke = Stroke::new(1.4, color);

    let half_w = (rect.width() * 0.35).min(5.0);
    let half_h = (rect.height() * 0.18).min(3.0);
    let c = rect.center();

    let (left, right, tip) = if is_popup_open {
        (
            egui::pos2(c.x - half_w, c.y + half_h * 0.5),
            egui::pos2(c.x + half_w, c.y + half_h * 0.5),
            egui::pos2(c.x, c.y - half_h * 1.5),
        )
    } else {
        (
            egui::pos2(c.x - half_w, c.y - half_h * 0.5),
            egui::pos2(c.x + half_w, c.y - half_h * 0.5),
            egui::pos2(c.x, c.y + half_h * 1.5),
        )
    };

    painter.line_segment([left, tip], stroke);
    painter.line_segment([tip, right], stroke);
}
