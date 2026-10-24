/// Events use original matrix coordinates. Eliminations apply only at their depth.
#[derive(Debug)]
pub enum Event {
    Evaluate {
        depth: usize,
        row: usize,
        col: usize,
        value: i128,
    },
    Eliminate {
        depth: usize,
        col: usize,
    },
    Level {
        depth: usize,
        rows: Vec<usize>,
        columns: Vec<usize>,
        recursive_rows: Vec<usize>,
    },
    Interpolate {
        depth: usize,
        row: usize,
        left: usize,
        right: usize,
    },
    Minimum {
        depth: usize,
        row: usize,
        col: usize,
    },
}

#[derive(Debug, Default)]
pub struct Trace {
    pub events: Vec<Event>,
}

impl Trace {
    /// Heap capacity estimate; excludes allocator metadata and stack frames.
    pub fn buffer_bytes(&self) -> usize {
        self.events.capacity() * std::mem::size_of::<Event>()
            + self
                .events
                .iter()
                .map(|event| match event {
                    Event::Level {
                        rows,
                        columns,
                        recursive_rows,
                        ..
                    } => {
                        (rows.capacity() + columns.capacity() + recursive_rows.capacity())
                            * std::mem::size_of::<usize>()
                    }
                    _ => 0,
                })
                .sum::<usize>()
    }
}
