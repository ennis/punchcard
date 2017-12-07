#![feature(box_syntax)]

extern crate zmq;
extern crate futures;
extern crate tokio_core;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rand;

use std::io;
use std::marker::Send;
use std::net::SocketAddr;

use futures::{future, Future};
use tokio_core::reactor::Core;
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

#[derive(Clone, Serialize, Deserialize)]
struct Entity
{
    name: String,
    id: i64
}

fn parse_rpc_method(path: &str) -> Option<(&str,&str)> {
    // skip leading '/'
    if !path.starts_with('/') {
        return None;
    }
    let inner_path = &path[1..];
    path.split('/').next()
}

trait RpcInterface {
    fn rpc(&self, id: &str, msg: &str) -> String;
    fn rpc_mut(&mut self, id: &str, msg: &str) -> String;
}

impl<T> RpcInterface for Vec<T> {
    fn rpc(&self, path: &str, msg: &str) -> String {
        if let Some((method,params)) = parse_rpc_method(path) {

        }
        String::new()
    }

    fn rpc_mut(&mut self, id: &str, msg: &str) -> String {
        String::new()
    }
}

fn main()
{
    const ENDPOINT: &str = "tcp://127.0.0.1:1234";
    let ctx = zmq::Context::new();
    let mut socket = ctx.socket(zmq::REP).unwrap();
    socket.bind(ENDPOINT).unwrap();
    println!("Listening on {}", ENDPOINT);

    loop {
        let msg = socket.recv_msg(0).expect("Failed to receive message");

        if let Some(s) = msg.as_str() {
            //println!("Received message: {}", s);
            if s.starts_with("entity:") {
                // This is a query, reply with some json
                let json = serde_json::to_string(&Entity { name: "test".to_owned(), id: rand::random() }).expect("Error serializing struct");
                socket.send(&json, 0);
            }
            else {
                // reply
                socket.send("YES!", 0);
            }

        }
        else {
            eprintln!("Message is not valid UTF-8");
        }
    }

    //socket.send_str("hello world!", 0).unwrap();


}
