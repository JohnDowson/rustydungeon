use rltk::RandomNumberGenerator as RNG;
use rltk::{GameState, Rltk, VirtualKeyCode, RGB, Point};
use specs::prelude::*;
// use specs_derive::Component;
mod rect;
pub use rect::*;
mod player;
use player::*;
mod map;
pub use map::*;
use std::{thread, time};
mod components;
pub use components::*;
mod visibility_system;
pub use visibility_system::VisibilitySystem;
mod ai_system;
pub use ai_system::*;
mod map_indexing_system;
use map_indexing_system::*;
mod combat_system;
use combat_system::*;
mod ui;

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    Paused,
    Running
}

pub struct State {
    pub ecs: World,
    pub rng: RNG,
    pub runstate: RunState
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        if self.runstate == RunState::Running {
            ctx.cls();
            ui::draw_ui(&self.ecs, ctx);
            self.run_systems();
            DamageSystem::delete_the_dead(&mut self.ecs);
            self.handle_input(ctx);

            let map = self.ecs.fetch::<Map>();
            map.draw_map(ctx);

            let positions = self.ecs.read_storage::<Position>();
            let renderables = self.ecs.read_storage::<Renderable>();

            for (pos, render) in (&positions, &renderables).join() {
                let idx = map.xy_idx(pos.x, pos.y);
                if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
                }
            }
            // TODO: something more robust, account for frametime
            thread::sleep(time::Duration::from_millis(60))
        } else {
            self.handle_input(ctx)
        }
    }
}
impl State {
    fn handle_input(&mut self, ctx: &mut Rltk) {
        if self.runstate == RunState::Paused {
            match ctx.key {
                None => {}
                Some(key) => match key {
                    VirtualKeyCode::R => self.regen_map(false),
                    VirtualKeyCode::F => self.reveal_all(),
                    VirtualKeyCode::P => self.toggle_runstate(ctx),
                    VirtualKeyCode::N => self.draw_pathing(ctx),
                    _ => {}
                }
            }
            return
        }
        match ctx.key {
            None => {}
            Some(key) => match key {
                VirtualKeyCode::Left => try_move_player(-1, 0, &mut self.ecs),
                VirtualKeyCode::Right => try_move_player(1, 0, &mut self.ecs),
                VirtualKeyCode::Up => try_move_player(0, -1, &mut self.ecs),
                VirtualKeyCode::Down => try_move_player(0, 1, &mut self.ecs),
                VirtualKeyCode::R => self.regen_map(false),
                VirtualKeyCode::T => self.regen_map(true),
                VirtualKeyCode::F => self.reveal_all(),
                VirtualKeyCode::P => self.toggle_runstate(ctx),
                _ => {}
            }
        }
    }
    fn toggle_runstate(&mut self, ctx: &mut Rltk) {
        match self.runstate {
            RunState::Running => { self.runstate = RunState::Paused; ctx.print(1, 1, "Paused") },
            RunState::Paused => { self.runstate = RunState::Running }
        }
    }
    fn draw_pathing(&mut self, ctx: &mut Rltk) {
        let map = self.ecs.fetch::<Map>();
        let (mut y, mut x) = (0, 0);
        for tile in map.blocked.iter() {
            let glyph;
                let fg;
                if *tile {
                        fg = RGB::from_f32(0.5, 0.0, 0.0);
                        glyph = rltk::to_cp437('#');
                    } else {
                        fg = RGB::from_f32(0.0, 0.5, 0.0);
                        glyph = rltk::to_cp437('#');
                    }
                ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);

            // Move the coordinates
            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mob = EnemyAI{};
        mob.run_now(&self.ecs);
        let mut mapindex = MapIndexingSystem{};
        mapindex.run_now(&self.ecs);
        let mut damage_system = DamageSystem{};
        damage_system.run_now(&self.ecs);
        let mut meele_system = MeleeCombatSystem{};
        meele_system.run_now(&self.ecs);
        self.ecs.maintain();
    }
    fn reveal_all(&mut self) {
        let mut map = self.ecs.fetch_mut::<Map>();
        map.revealed_tiles = vec![true; 80*50];
    }
    fn regen_map(&mut self, test: bool) {
        self.ecs.remove::<Map>();
        let map = if test {
            Map::test_map()
        } else {
            Map::new_map(self.rng.range(0 as u64, 9999 as u64))
        };
        self.ecs.insert(map);
        let map = self.ecs.fetch::<Map>();
        let mut positions = self.ecs.write_storage::<Position>();
        let mut players = self.ecs.write_storage::<Player>();
        for (_player, pos) in (&mut players, &mut positions).join() {
            let room0 = map.rooms[0].center();
            pos.x = room0.0;
            pos.y = room0.1;
        }
    }
}

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Rusty Dungeon")
        .build()?;
    let mut gs = State { ecs: World::new(), rng: RNG::new(), runstate: RunState::Running};
    /*TODO: somehow refactor this to form of
    use components
    ...
    for component register <component>
    */
    //let map = Map::new_map(0);
    let map = Map::test_map();
    let player_spawn_room = gs.rng.range(0 as usize, map.rooms_n());
    // let player_spawn_pos = map.rooms[player_spawn_room].center();
    let player_spawn_pos = (10, 10);
    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Enemy>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<SufferDamage>();
    gs.ecs.register::<WantsToMelee>();
    // enemy
    for (idx, room) in map.rooms.iter().enumerate() {
        if idx != player_spawn_room {
        let (x,y) = room.center();
        gs.ecs.create_entity()
            .with(Position{ x, y })
            .with(Renderable{
                glyph: rltk::to_cp437('E'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Enemy{})
            .with(BlocksTile{})
            .with(CombatStats{ max_hp: 5, hp: 5, defense: 1, power: 1 })
            .with(Name { name: format!("E#{}", idx)})
            .build();
        } else {}
    }
    // test enemy
    gs.ecs.create_entity()
            .with(Position{ x:5, y:5 })
            .with(Renderable{
                glyph: rltk::to_cp437('R'),
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
            .with(Enemy{})
            .with(BlocksTile{})
            .with(CombatStats{ max_hp: 5, hp: 5, defense: 1, power: 1 })
            .with(Name { name: format!("R#{}", 0)})
            .build();
    // player
    let player_entity = gs.ecs
        .create_entity()
        .with(Position::from_tuple(player_spawn_pos))
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Viewshed::new(8))
        .with(Player {})
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 20, hp: 20, defense: 1, power: 3 })
        .with(Name { name: "Player".to_string()})
        .build();
    gs.ecs.insert(player_entity);
    gs.ecs.insert(Point::from_tuple(player_spawn_pos));
    gs.ecs.insert(map);
    // create mor entities here
    rltk::main_loop(context, gs)
}
