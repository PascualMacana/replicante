//! Plato de Petri: el padre se queda y brota una hija.
//! Es la imagen de `spawn`, sin escribir archivos.

use std::io::{self, Write};
use std::thread;
use std::time::Duration;

const WIDTH: usize = 63;
const HEIGHT: usize = 21;
const MAX_CELLS: usize = 12;
const BUD_FRAMES: u32 = 16;

pub fn run(
    origin_gen: u32,
    origin_lineage: &str,
    splits: u32,
    delay_ms: u64,
) -> io::Result<()> {
    let _cursor = HideCursor::new();
    let slots = slots();
    let mut cells = vec![Cell {
        x: slots[0].0,
        y: slots[0].1,
        generation: origin_gen,
        lineage: origin_lineage.to_string(),
        buds: 0,
    }];
    let mut occupied = vec![false; slots.len()];
    occupied[0] = true;

    draw_scene(&cells, None, splits, 0)?;
    sleep(delay_ms.saturating_mul(4));

    let mut done: u32 = 0;
    while done < splits && cells.len() < MAX_CELLS {
        let Some((pi, dest_slot)) = pick_bud(&cells, &occupied, &slots) else {
            break;
        };
        let parent = cells[pi].clone();
        let dest = slots[dest_slot];
        let d_gen = parent.generation + 1;
        let d_lin = format!("{}.{}", parent.lineage, parent.buds + 1);
        cells[pi].buds += 1;

        for frame in 1..=BUD_FRAMES {
            let bud = Some(Bud {
                parent: parent.clone(),
                dest,
                frame,
                d_gen,
                d_lin: d_lin.clone(),
            });
            draw_scene(&cells, bud.as_ref(), splits, done)?;
            sleep(delay_ms);
        }

        occupied[dest_slot] = true;
        cells.push(Cell {
            x: dest.0,
            y: dest.1,
            generation: d_gen,
            lineage: d_lin,
            buds: 0,
        });
        done += 1;
        draw_scene(&cells, None, splits, done)?;
        sleep(delay_ms.saturating_mul(3));
    }

    Ok(())
}

#[derive(Clone)]
struct Cell {
    x: i32,
    y: i32,
    generation: u32,
    lineage: String,
    buds: u32,
}

struct Bud {
    parent: Cell,
    dest: (i32, i32),
    frame: u32,
    d_gen: u32,
    d_lin: String,
}

struct HideCursor;

impl HideCursor {
    fn new() -> Self {
        print!("\x1b[?25l\x1b[2J");
        let _ = io::stdout().flush();
        HideCursor
    }
}

impl Drop for HideCursor {
    fn drop(&mut self) {
        print!("\x1b[?25h\x1b[0m\n");
        let _ = io::stdout().flush();
    }
}

fn sleep(ms: u64) {
    if ms > 0 {
        thread::sleep(Duration::from_millis(ms));
    }
}

fn slots() -> Vec<(i32, i32)> {
    let (cx, cy) = (31_i32, 10_i32);
    vec![
        (cx, cy),
        (cx + 14, cy),
        (cx - 14, cy),
        (cx + 8, cy + 5),
        (cx - 8, cy + 5),
        (cx + 8, cy - 5),
        (cx - 8, cy - 5),
        (cx + 22, cy + 3),
        (cx - 22, cy + 3),
        (cx + 22, cy - 3),
        (cx - 22, cy - 3),
        (cx, cy + 7),
    ]
}

fn pick_bud(
    cells: &[Cell],
    occupied: &[bool],
    slots: &[(i32, i32)],
) -> Option<(usize, usize)> {
    let mut order: Vec<usize> = (0..cells.len()).collect();
    order.sort_by_key(|&i| (cells[i].buds, i));
    for pi in order {
        let parent = &cells[pi];
        let mut best: Option<(usize, i32)> = None;
        for (si, slot) in slots.iter().enumerate() {
            if occupied[si] {
                continue;
            }
            let d = (slot.0 - parent.x).pow(2) + (slot.1 - parent.y).pow(2) * 4;
            if best.map(|(_, bd)| d < bd).unwrap_or(true) {
                best = Some((si, d));
            }
        }
        if let Some((si, _)) = best {
            return Some((pi, si));
        }
    }
    None
}

fn draw_scene(
    cells: &[Cell],
    bud: Option<&Bud>,
    goal: u32,
    done: u32,
) -> io::Result<()> {
    let mut c = Canvas::new(WIDTH, HEIGHT);
    draw_dish(&mut c);

    for cell in cells {
        put_cell(&mut c, cell.x, cell.y, 6.0, 3.0, cell.generation);
    }

    if let Some(bud) = bud {
        let t = bud.frame as f32 / BUD_FRAMES as f32;
        let x = lerp(bud.parent.x as f32, bud.dest.0 as f32, t.powf(0.85));
        let y = lerp(bud.parent.y as f32, bud.dest.1 as f32, t.powf(0.85));
        let rx = lerp(2.5, 6.0, t);
        let ry = lerp(1.5, 3.0, t);
        put_cell(&mut c, x.round() as i32, y.round() as i32, rx, ry, bud.d_gen);
        let _ = &bud.d_lin;
    }

    let status = if bud.is_some() {
        "budding"
    } else if done >= goal || cells.len() >= MAX_CELLS {
        "done"
    } else {
        "idle"
    };
    present(&c, cells, status, done, goal)
}

fn draw_dish(c: &mut Canvas) {
    let cx = (WIDTH as i32) / 2;
    let cy = (HEIGHT as i32) / 2;
    let rx = (WIDTH as i32) / 2 - 1;
    let ry = (HEIGHT as i32) / 2 - 1;
    for y in 0..HEIGHT as i32 {
        for x in 0..WIDTH as i32 {
            if on_ellipse(x, y, cx, cy, rx, ry, 1.4) {
                c.put(x, y, '·', 90);
            }
        }
    }
}

fn put_cell(c: &mut Canvas, cx: i32, cy: i32, rx: f32, ry: f32, gen: u32) {
    let color = 36 - (gen % 6) as u8;
    let color = match color {
        36 => 36,
        35 => 32,
        34 => 33,
        33 => 35,
        32 => 34,
        _ => 31,
    };
    let rx_i = rx.max(2.0);
    let ry_i = ry.max(1.0);
    for y in (cy - ry_i.ceil() as i32)..=(cy + ry_i.ceil() as i32) {
        for x in (cx - rx_i.ceil() as i32)..=(cx + rx_i.ceil() as i32) {
            let dx = (x as f32 - cx as f32) / rx_i;
            let dy = (y as f32 - cy as f32) / ry_i;
            let d = dx * dx + dy * dy;
            if d > 1.0 {
                continue;
            }
            let membrane = d > 0.62;
            let ch = if membrane { 'o' } else { ':' };
            c.put(x, y, ch, color);
        }
    }
    let label = if gen < 10 {
        gen.to_string()
    } else {
        format!("{}", gen % 100)
    };
    let start = cx - (label.len() as i32) / 2;
    for (i, ch) in label.chars().enumerate() {
        c.put(start + i as i32, cy, ch, color);
    }
}

fn on_ellipse(x: i32, y: i32, cx: i32, cy: i32, rx: i32, ry: i32, band: f32) -> bool {
    let dx = (x - cx) as f32 / rx as f32;
    let dy = (y - cy) as f32 / ry as f32;
    let d = dx * dx + dy * dy;
    (d - 1.0).abs() < (band / rx.max(1) as f32)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

struct Canvas {
    w: usize,
    h: usize,
    ch: Vec<char>,
    fg: Vec<u8>,
}

impl Canvas {
    fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            ch: vec![' '; w * h],
            fg: vec![0; w * h],
        }
    }

    fn put(&mut self, x: i32, y: i32, ch: char, color: u8) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        if x >= self.w || y >= self.h {
            return;
        }
        let i = y * self.w + x;
        self.ch[i] = ch;
        self.fg[i] = color;
    }
}

fn present(
    c: &Canvas,
    cells: &[Cell],
    status: &str,
    done: u32,
    goal: u32,
) -> io::Result<()> {
    let mut out = String::with_capacity(c.w * c.h * 8);
    out.push_str("\x1b[H");
    out.push_str("\x1b[0m  replicante  ·  petri dish\x1b[K\n");
    for y in 0..c.h {
        let mut last = 255_u8;
        for x in 0..c.w {
            let i = y * c.w + x;
            if c.fg[i] != last {
                if c.fg[i] == 0 {
                    out.push_str("\x1b[0m");
                } else {
                    out.push_str(&format!("\x1b[{}m", c.fg[i]));
                }
                last = c.fg[i];
            }
            out.push(c.ch[i]);
        }
        out.push_str("\x1b[0m\x1b[K\n");
    }
    out.push_str("\x1b[0m");
    out.push_str(&format!(
        "  {status}  ·  {} cells  ·  {done}/{goal} buds\x1b[K\n",
        cells.len()
    ));
    if let Some(last) = cells.last() {
        out.push_str(&format!(
            "  newest  gen {}  lineage {}\x1b[K\n",
            last.generation, last.lineage
        ));
    }
    out.push_str("  parent stays; a daughter pinches off — that's spawn\x1b[K\n");
    let mut stdout = io::stdout();
    stdout.write_all(out.as_bytes())?;
    stdout.flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_are_unique() {
        let s = slots();
        let mut t = s.clone();
        t.sort();
        t.dedup();
        assert_eq!(s.len(), t.len());
        assert!(s.len() >= MAX_CELLS);
    }

    #[test]
    fn first_bud_goes_next_to_origin() {
        let slots = slots();
        let cells = vec![Cell {
            x: slots[0].0,
            y: slots[0].1,
            generation: 0,
            lineage: "0".into(),
            buds: 0,
        }];
        let mut occ = vec![false; slots.len()];
        occ[0] = true;
        let (pi, si) = pick_bud(&cells, &occ, &slots).unwrap();
        assert_eq!(pi, 0);
        assert_ne!(si, 0);
        let d2 = (slots[si].0 - slots[0].0).pow(2) + (slots[si].1 - slots[0].1).pow(2);
        assert!(d2 < 20 * 20);
    }

    #[test]
    fn lineage_matches_spawn() {
        assert_eq!(format!("{}.{}", "0", 1), "0.1");
        assert_eq!(format!("{}.{}", "0.1", 2), "0.1.2");
    }
}
