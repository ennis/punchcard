use std::collections::HashMap;
use std::net;
use std::cell::{RefCell, Cell, Ref};
use std::rc::Rc;
use std::clone::Clone;
use std::ops::{Add};

type WidgetID = i32;


///////////////////////////////////////////////////////////////
// Common traits
trait WidgetContainer
{
    fn button(&self) -> Button {
        Button { id: 0 }
    }

    fn labeled_button(&self, label: &str) -> Button {
        let b = Button { id: 0 };
        b.label(label);
        b
    }

    fn fieldset(&self) -> FieldSet {
        FieldSet { id: 0 }
    }
}

trait HasLabel
{
    fn label(&self, label: &str) -> &Self;
}

trait HasGraphic
{
    fn graphic(&self) -> &Self;
}


///////////////////////////////////////////////////////////////
// Ui
struct Ui
{
}

impl Ui
{
    pub fn new() -> Ui {
        Ui {}
    }
}

impl WidgetContainer for Ui
{
}

///////////////////////////////////////////////////////////////
// Button
struct Button
{
    id: WidgetID
}

impl Button
{
    pub fn on_action<F: Fn(&Self)>(&self, f: F) {
        f(self)
    }
}

impl WidgetContainer for Button {
}

impl HasLabel for Button
{
    fn label(&self, label: &str) -> &Self {
        self
    }
}



///////////////////////////////////////////////////////////////
// FieldSet
struct FieldSet
{
    id: WidgetID
}

impl FieldSet
{
    pub fn field(&self, label: &str) -> Field {
        Field { id: 0 }
    }
}

impl WidgetContainer for FieldSet {
}

impl HasLabel for FieldSet
{
    fn label(&self, label: &str) -> &Self {
        self
    }
}


///////////////////////////////////////////////////////////////
// Field
struct Field
{
    id: WidgetID
}

impl Field
{
}


///////////////////////////////////////////////////////////////
// Observable traits
trait Observable
{
    // everything in f should outlive self: use only values? use 'static?
    fn observe<F: Fn(&Self) + 'static>(&self, f: F);
}

trait ObservableValue<T>: Observable
{
    fn observe_value<F: Fn(&Self, T) + 'static>(&self, f: F);
    /// Returns a copy of the current value
    fn value(&self) -> T;
}

trait WritableValue<T>
{
    fn set_value(&self, v: T);
}

trait PropertyBase<T>: ObservableValue<T> + WritableValue<T>
{
    fn bind<S: ObservableValue<T>>(&self, src: S);
}

// now for the implementation:
// use Arc everywhere I guess...
// but hide internally

// Properties should have pointer semantics
struct PropertyCell<T>
{
    listeners: RefCell<Vec<Box<Fn() + 'static>>>,
    value: RefCell<T>
}

impl<T: 'static> PropertyCell<T> {
    pub fn new(value: T) -> PropertyCell<T> {
        PropertyCell {
            listeners: RefCell::new(Vec::new()),
            value: RefCell::new(value)
        }
    }

    // XXX notify should send a copy to every listener?
    pub fn notify(&self)
    {
        for li in self.listeners.borrow().iter() {
            li();
        }
    }
}


struct Property<T: 'static>
{
    inner: Rc<PropertyCell<T>>
}

impl<T: 'static> Clone for Property<T> {
    fn clone(&self) -> Self {
        Property {
            inner: self.inner.clone()
        }
    }
}

impl<T: Clone + 'static> Property<T>
{
    pub fn new(initial_value: T) -> Property<T> {
        Property {
            inner: Rc::new(PropertyCell::new(initial_value))
        }
    }
}

impl<T: Clone + 'static> Property<T>
{
    pub fn bind(&self, other: &Property<T>) {
        let clone = self.clone();
        other.observe_value(move |_, val| {
            clone.set_value(val)
        });
    }
}

impl<T: 'static> Observable for Property<T> {
    fn observe<F: Fn(&Self) + 'static>(&self, f: F) {
        let clone = (*self).clone();
        self.inner.listeners.borrow_mut().push(Box::new(move || {
            f(&clone);
        }));
    }
}

impl<T: Clone + 'static> ObservableValue<T> for Property<T> {
    fn observe_value<F: Fn(&Self, T) + 'static>(&self, f: F) {
        let clone = (*self).clone();
        self.inner.listeners.borrow_mut().push(Box::new(move || {
            let v = clone.value();
            f(&clone, v);
        }));
    }

    fn value(&self) -> T {
        self.inner.value.borrow().clone()
    }
}

impl <T: 'static> WritableValue<T> for Property<T> {
    fn set_value(&self, v: T) {
        *self.inner.value.borrow_mut() = v;
        self.inner.notify();
    }
}

impl <T: Add + Clone + 'static> Add for Property<T>
    where <T as Add>::Output: Clone
{
    type Output = Property<<T as Add>::Output>;

    fn add(self, rhs: Property<T>) -> Self::Output {
        let p = Property::new(self.value() + rhs.value());
        let cp = p.clone();
        self.observe(move |it| {
            cp.set_value(it.value() + rhs.value())
        });
        rhs.observe(move |it| {
            cp.set_value(self.value() + it.value())
        });
        p
    }
}

fn main()
{
    let p = Property::new(10i32);
    p.observe_value(|it, new| {
       println!("New value: {}", new);
    });
    p.set_value(12i32);

    let string_prop = Property::new("ddsfs");
    string_prop.observe_value(|it, new| {
        println!("New value: {}", new);
    });
    string_prop.set_value("fgdojgfd");

    let string_prop2 = Property::new("msglmkg");
    let string_prop3 = Property::new("fsmdfdsm");
    string_prop2.bind(&string_prop);
    string_prop3.bind(&string_prop);
    string_prop.set_value("fsmdfsmdlfksd");
    println!("{}", string_prop2.value());
    println!("{}", string_prop3.value());
}
