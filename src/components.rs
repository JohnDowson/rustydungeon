use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use rltk::RandomNumberGenerator as RNG;
use specs::prelude::*;
use specs_derive::Component;
use crate::rect::*;

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl Position {
    pub fn from_tuple((x, y) : (i32, i32)) -> Position {
        Position { x: x, y: y }
    }
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component)]
pub struct LeftMover {}

pub struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);
    fn run(&mut self, (lefty, mut pos) : Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x <0 { pos.x = 79}
        }
    }
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<rltk::Point>,
    pub range: i32
}
impl Viewshed {
    pub fn new(range: i32) -> Viewshed {
        Viewshed { visible_tiles: Vec::new(), range: range}
    }
}

#[derive(Component, Debug)]
pub struct Player {}