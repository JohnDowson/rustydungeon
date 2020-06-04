use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use rltk::RandomNumberGenerator as RNG;
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};
mod rect;
use rect::*;
mod map;
pub use map::*;

// TODO: move components to a separate file/files
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component)]
struct LeftMover {}

struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);
    fn run(&mut self, (lefty, mut pos) : Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x <0 { pos.x = 79}
        }
    }
}

#[derive(Component, Debug)]
struct Player {}

struct State {
    ecs: World,
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        
        self.run_systems();
        self.handle_input(ctx);

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
impl State {
    fn handle_input(&mut self, ctx: &mut Rltk) {
        match ctx.key {
            None => {}
            Some(key) => match key {
                VirtualKeyCode::Left => try_move_player(-1, 0, &mut self.ecs),
                VirtualKeyCode::Right => try_move_player(1, 0, &mut self.ecs),
                VirtualKeyCode::Up => try_move_player(0, -1, &mut self.ecs),
                VirtualKeyCode::Down => try_move_player(0, 1, &mut self.ecs),
                _ => {}
            }
        }
    }
    fn run_systems(&mut self) {
        let mut lw = LeftWalker{};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[destination_idx] != TileType::Wall {
            pos.x = min(79 , max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Rusty Dungeon")
        .build()?;
    let mut gs = State { ecs: World::new() };
    /*TODO: somehow refactor this to form of
    use components
    ...
    for component register <component>
    */
    let mut rng = RNG::new();
    let (map, rooms) = new_map(0);
    let player_spawn = rooms[rng.range(0 as usize, rooms.len())].center();
    gs.ecs.insert(map);
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    gs.ecs
        .create_entity()
        .with(Position { x: player_spawn.0, y: player_spawn.1 })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player{})
        .build();
    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: rltk::to_cp437('☺'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(LeftMover {})
            .build();
    }
    rltk::main_loop(context, gs)
}
