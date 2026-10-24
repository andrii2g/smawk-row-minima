use crate::{Event, Matrix, SearchResult, Trace};
use std::fmt::Write;

pub struct SvgDiagram {
    pub svg: String,
    /// Rendering evaluations are separate from the solver's measurements.
    pub evaluations: usize,
}

/// Render a small matrix and depth-local elimination strips. Accepts at most
/// 32 rows and columns; the trace and result must come from this matrix.
pub fn render_svg<M: Matrix>(
    matrix: &M,
    result: &SearchResult,
    trace: &Trace,
) -> Result<SvgDiagram, &'static str> {
    let (m, n) = (matrix.rows(), matrix.cols());
    if m == 0 || n == 0 || m > 32 || n > 32 {
        return Err("SVG requires 1..=32 rows and columns");
    }
    if result.minima.len() != m || result.minima.iter().any(|&c| c >= n) {
        return Err("invalid minima for SVG dimensions");
    }
    let levels: Vec<_> = trace
        .events
        .iter()
        .filter_map(|event| match event {
            Event::Level {
                depth,
                rows,
                columns,
                recursive_rows,
            } => Some((*depth, rows, columns, recursive_rows)),
            _ => None,
        })
        .collect();
    let width = (n * 34 + 120).max(920);
    let base = 190 + m * 34;
    let height = base + 90 + levels.len() * 132;
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}" role="img" aria-labelledby="title desc">
<title id="title">SMAWK row minima: {m} by {n}</title>
<desc id="desc">Heatmap of matrix values. Dots mark evaluated cells; green outlines mark leftmost minima. Recursion panels show depth-local column elimination and selected rows.</desc>
<rect width="100%" height="100%" fill="#0f172a"/>
<style>text{{font-family:Consolas,monospace;fill:#e2e8f0;font-size:12px}} .heading{{font-size:24px;font-weight:bold}} .small{{font-size:11px}}</style>
<text x="28" y="38" class="heading">SMAWK / ROW MINIMA</text>
<text x="28" y="65">{m} rows x {n} columns | {} evaluations | {} comparisons | depth {}</text>
<text x="28" y="89">Dot: evaluated (tooltip: visits) | Green outline: leftmost minimum</text>
<text x="28" y="111">Heat: low = dark, high = light | Background rendering: {} separate evaluations</text>
"##,
        result.metrics.evaluations,
        result.metrics.comparisons,
        result.metrics.recursion_depth,
        m * n
    );
    let values: Vec<_> = (0..m)
        .flat_map(|r| (0..n).map(move |c| matrix.at(r, c)))
        .collect();
    let low = *values.iter().min().unwrap() as f64;
    let high = *values.iter().max().unwrap() as f64;
    let mut visits = vec![0usize; m * n];
    for event in &trace.events {
        if let Event::Evaluate { row, col, .. } = *event {
            if row >= m || col >= n {
                return Err("trace coordinates exceed SVG dimensions");
            }
            visits[row * n + col] += 1;
        }
    }
    for c in 0..n {
        writeln!(
            svg,
            "<text x=\"{}\" y=\"147\" text-anchor=\"middle\">{c}</text>",
            77 + c * 34
        )
        .unwrap();
    }
    for r in 0..m {
        writeln!(
            svg,
            "<text x=\"42\" y=\"{}\" text-anchor=\"end\">{r}</text>",
            179 + r * 34
        )
        .unwrap();
        for c in 0..n {
            let value = values[r * n + c];
            let t = if high == low {
                0.0
            } else {
                (value as f64 - low) / (high - low)
            };
            let lightness = 22.0 + t * 46.0;
            let (x, y) = (60 + c * 34, 157 + r * 34);
            writeln!(svg, "<rect x=\"{x}\" y=\"{y}\" width=\"32\" height=\"32\" rx=\"3\" fill=\"hsl(215,55%,{lightness:.1}%)\"><title>row {r}, column {c}: {value}; evaluations {}</title></rect>",visits[r*n+c]).unwrap();
            if visits[r * n + c] > 0 {
                writeln!(
                    svg,
                    "<circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"#f8fafc\"/>",
                    x + 16,
                    y + 16
                )
                .unwrap();
            }
            if result.minima[r] == c {
                writeln!(svg, "<rect x=\"{x}\" y=\"{y}\" width=\"32\" height=\"32\" rx=\"3\" fill=\"none\" stroke=\"#4ade80\" stroke-width=\"3\"/>").unwrap();
            }
        }
    }
    let points = result
        .minima
        .iter()
        .enumerate()
        .map(|(r, c)| format!("{},{}", 76 + c * 34, 173 + r * 34))
        .collect::<Vec<_>>()
        .join(" ");
    writeln!(svg, "<polyline points=\"{points}\" fill=\"none\" stroke=\"#4ade80\" stroke-width=\"2\" opacity=\"0.7\"/>").unwrap();
    writeln!(svg, "<text x=\"28\" y=\"{base}\">RECURSION | red: eliminated here | blue: retained | purple: recurse | gray: inactive</text>").unwrap();
    let mut candidates: Vec<_> = (0..n).collect();
    for (index, (depth, rows, columns, recursive_rows)) in levels.iter().enumerate() {
        let y = base + 32 + index * 132;
        writeln!(
            svg,
            "<text x=\"28\" y=\"{y}\">Depth {depth}: {} active rows; {} -&gt; {} columns</text>",
            rows.len(),
            candidates.len(),
            columns.len()
        )
        .unwrap();
        for (label, count, offset) in [("cols", n, 16), ("rows", m, 54)] {
            writeln!(
                svg,
                "<text x=\"28\" y=\"{}\">{label}</text>",
                y + offset + 19
            )
            .unwrap();
            for coordinate in 0..count {
                let color = if label == "cols" {
                    if columns.contains(&coordinate) {
                        "#2563eb"
                    } else if candidates.contains(&coordinate) {
                        "#b91c1c"
                    } else {
                        "#334155"
                    }
                } else if recursive_rows.contains(&coordinate) {
                    "#7c3aed"
                } else if rows.contains(&coordinate) {
                    "#2563eb"
                } else {
                    "#334155"
                };
                let x = 76 + coordinate * 24;
                writeln!(svg, "<rect x=\"{x}\" y=\"{}\" width=\"22\" height=\"28\" rx=\"3\" fill=\"{color}\"/><text x=\"{}\" y=\"{}\" text-anchor=\"middle\" class=\"small\">{coordinate}</text>",y+offset,x+11,y+offset+19).unwrap();
            }
        }
        candidates = columns.to_vec();
    }
    svg.push_str("</svg>\n");
    Ok(SvgDiagram {
        svg,
        evaluations: m * n,
    })
}
