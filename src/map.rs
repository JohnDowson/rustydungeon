use crate::rect::*;
use rltk::RandomNumberGenerator as RNG;
use rltk::{Algorithm2D, BaseMap, Point, Rltk, RGB};
use std::cmp::{max, min};
use specs::prelude::*;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content : Vec<Vec<Entity>>
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        ((y * self.width) + x) as usize
    }
    pub fn rooms_n(&self) -> usize {
        self.rooms.len()
    }

    pub fn new_map(seed: u64) -> Map {
        let mut rng = RNG::seeded(seed);
        let mut map = Map {
            tiles: vec![TileType::Wall; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false; 80 * 50],
            visible_tiles: vec![false; 80 * 50],
            blocked: vec![false; 80*50],
            tile_content : vec![Vec::new(); 80*50]
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, 80 - w - 1) - 1;
            let y = rng.roll_dice(1, 50 - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) {
                    ok = false
                }
            }
            if ok {
                map.add_room(&new_room);
                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.add_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.add_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.add_vertical_tunnel(prev_y, new_y, prev_x);
                        map.add_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        return map;
    }

    fn add_room(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn add_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn add_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < 80 * 50 {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn draw_map(&self, ctx: &mut Rltk) {
        let (mut y, mut x) = (0, 0);
        for (idx, tile) in self.tiles.iter().enumerate() {
            if self.revealed_tiles[idx] {
                let glyph;
                let mut fg;
                match tile {
                    TileType::Floor => {
                        fg = RGB::from_f32(0.5, 0.5, 0.5);
                        glyph = rltk::to_cp437('.');
                    }
                    TileType::Wall => {
                        fg = RGB::from_f32(0.0, 1.0, 0.0);
                        glyph = rltk::to_cp437('#');
                    }
                }
                if !self.visible_tiles[idx] {
                    fg = fg.to_greyscale();
                }
                ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
            }
            // Move the coordinates
            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }

    fn is_valid_exit(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false
        } else {
        let idx = self.xy_idx(x, y);
        return !self.blocked[idx]
        }
    }

    pub fn test_map() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Floor; 80 * 50],
            rooms: Vec::new(),
            width: 80,
            height: 50,
            revealed_tiles: vec![false; 80 * 50],
            visible_tiles: vec![false; 80 * 50],
            blocked: vec![false; 80*50],
            tile_content : vec![Vec::new(); 80*50]
        };
        map.rooms.push(Rect::new(10, 10, 10, 10));
        for x in 0..80 {
            let idx = map.xy_idx(x, 0);
            map.tiles[idx] = TileType::Wall;
            let idx = map.xy_idx(x, 49);
            map.tiles[idx] = TileType::Wall;
        }
        for y in 0..50 {
            let idx = map.xy_idx(0, y);
            map.tiles[idx] = TileType::Wall;
            let idx = map.xy_idx(79, y);
            map.tiles[idx] = TileType::Wall;
        }
        let mut rng = RNG::new();
        for _i in 0..400 {
            let x = rng.roll_dice(1, 79);
            let y = rng.roll_dice(1, 49);
            let idx = map.xy_idx(x, y);
            if idx != map.xy_idx(40, 25) {
                map.tiles[idx] = TileType::Wall;
            }
        }
        return map;
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits =rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.height;
        let w = self.width as usize;
        if self.is_valid_exit(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_valid_exit(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_valid_exit(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_valid_exit(x, y+1) { exits.push((idx+w, 1.0)) };

        /*
        if self.is_valid_exit(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.is_valid_exit(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.is_valid_exit(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.is_valid_exit(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }
        */
        exits
    }
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

/*pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        // Render a tile depending upon the tile type
        match tile {
            TileType::Floor => {
                ctx.set(x, y, RGB::from_f32(0.5, 0.5, 0.5), RGB::from_f32(0., 0., 0.), rltk::to_cp437('.'));
            }
            TileType::Wall => {
                ctx.set(x, y, RGB::from_f32(0.0, 1.0, 0.0), RGB::from_f32(0., 0., 0.), rltk::to_cp437('#'));
            }
        }

        // Move the coordinates
        x += 1;
        if x > 79 {
            x = 0;
            y += 1;
        }
    }
} */
