use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use rltk::RandomNumberGenerator as RNG;
use specs::prelude::*;
use specs_derive::Component;
use std::cmp::{max, min};
mod rect;
pub use rect::*;
mod map;
pub use map::*;
use std::{thread, time};
mod components;
pub use components::*;

struct State {
    ecs: World,
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        
        self.run_systems();
        self.handle_input(ctx);

        let map = self.ecs.fetch::<Map>();
        draw_map(&map.tiles, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
        // TODO: something more robust, account for frametime
        thread::sleep(time::Duration::from_millis(33))
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
    let map = ecs.fetch::<Map>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map.tiles[destination_idx] != TileType::Wall {
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
    let map = Map::new_map(0);
    let player_spawn = map.rooms[rng.range(0 as usize, map.rooms_n())].center();
    gs.ecs.insert(map);
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs
        .create_entity()
        .with(Position::from_tuple(player_spawn))
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed::new(8))
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
