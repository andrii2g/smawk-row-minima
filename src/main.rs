use smawk_row_minima::{
    MongeMatrix, SearchResult, brute_force, is_monge, smawk, smawk_traced, svg::render_svg,
};
use std::{collections::BTreeMap, error::Error, path::Path};

const HELP: &str = "SMAWK row minima (all results verified against brute force)
Usage:
  smawk-row-minima verify [--rows 100] [--cols 150] [--seed 42]
  smawk-row-minima visualize [--rows 12] [--cols 18] [--seed 42] [--out artifacts/example.svg]
  smawk-row-minima bench [--sizes 64,256,1024] [--seed 42] [--repeats 5]
Bench measures square, tall and wide shapes, reporting median search times as CSV.
SVG dimensions must each be 1..=32. Monge checks run when both dimensions <=16.
";

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn number(
    options: &BTreeMap<String, String>,
    key: &str,
    default: &str,
) -> Result<usize, Box<dyn Error>> {
    Ok(options.get(key).map_or(default, String::as_str).parse()?)
}

fn verify_pair(matrix: &MongeMatrix) -> Result<(SearchResult, SearchResult), Box<dyn Error>> {
    let brute = brute_force(matrix)?;
    let fast = smawk(matrix)?;
    if brute.minima != fast.minima {
        return Err("SMAWK disagrees with brute force".into());
    }
    Ok((brute, fast))
}

fn report(name: &str, result: &SearchResult) {
    let m = &result.metrics;
    println!(
        "{name}: evaluations={} comparisons={} eliminated={} depth={} runtime_ns={} buffer_bytes={}",
        m.evaluations,
        m.comparisons,
        m.eliminated_columns,
        m.recursion_depth,
        m.elapsed.as_nanos(),
        m.buffer_bytes
    );
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "--help".into());
    if command == "--help" || command == "-h" {
        print!("{HELP}");
        return Ok(());
    }
    let allowed: &[&str] = match command.as_str() {
        "verify" => &["--rows", "--cols", "--seed"],
        "visualize" => &["--rows", "--cols", "--seed", "--out"],
        "bench" => &["--sizes", "--seed", "--repeats"],
        _ => return Err(format!("unknown command: {command}; use --help").into()),
    };
    let mut options = BTreeMap::new();
    while let Some(flag) = args.next() {
        if flag == "--help" {
            print!("{HELP}");
            return Ok(());
        }
        if !allowed.contains(&flag.as_str()) {
            return Err(format!("unknown option: {flag}").into());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("missing value for {flag}"))?;
        if options.insert(flag.clone(), value).is_some() {
            return Err(format!("duplicate option: {flag}").into());
        }
    }
    let seed: u64 = options.get("--seed").map_or("42", String::as_str).parse()?;
    if command == "bench" {
        let sizes: Vec<usize> = options
            .get("--sizes")
            .map_or("64,256,1024", String::as_str)
            .split(',')
            .map(str::parse)
            .collect::<Result<_, _>>()?;
        let repeats = number(&options, "--repeats", "5")?;
        if repeats == 0 || sizes.contains(&0) {
            return Err("sizes and repeats must be positive".into());
        }
        println!(
            "rows,cols,seed,repeats,algorithm,evaluations,comparisons,eliminated_columns,recursion_depth,median_ns,buffer_bytes"
        );
        for size in sizes {
            for (rows, cols) in [
                (size, size),
                (size, (size / 8).max(1)),
                ((size / 8).max(1), size),
            ] {
                let matrix = MongeMatrix::seeded(rows, cols, seed);
                verify_pair(&matrix)?; // Untimed warm-up, also verifies correctness.
                let mut timings = [Vec::new(), Vec::new()];
                let mut last = None;
                for iteration in 0..repeats {
                    // Alternate execution order to reduce systematic cache/order bias.
                    let (brute, fast) = if iteration % 2 == 0 {
                        verify_pair(&matrix)?
                    } else {
                        let fast = smawk(&matrix)?;
                        let brute = brute_force(&matrix)?;
                        if fast.minima != brute.minima {
                            return Err("SMAWK disagrees with brute force".into());
                        }
                        (brute, fast)
                    };
                    timings[0].push(brute.metrics.elapsed.as_nanos());
                    timings[1].push(fast.metrics.elapsed.as_nanos());
                    last = Some((brute, fast));
                }
                let (brute, fast) = last.unwrap();
                for (index, (name, result)) in [("brute_force", brute), ("smawk", fast)]
                    .into_iter()
                    .enumerate()
                {
                    timings[index].sort_unstable();
                    let median = timings[index][repeats / 2];
                    let m = result.metrics;
                    println!(
                        "{rows},{cols},{seed},{repeats},{name},{},{},{},{},{median},{}",
                        m.evaluations,
                        m.comparisons,
                        m.eliminated_columns,
                        m.recursion_depth,
                        m.buffer_bytes
                    );
                }
            }
        }
        return Ok(());
    }
    let visual = command == "visualize";
    let rows = number(&options, "--rows", if visual { "12" } else { "100" })?;
    let cols = number(&options, "--cols", if visual { "18" } else { "150" })?;
    if visual && (rows == 0 || cols == 0 || rows > 32 || cols > 32) {
        return Err("SVG requires 1..=32 rows and columns".into());
    }
    let matrix = MongeMatrix::seeded(rows, cols, seed);
    let (brute, fast) = verify_pair(&matrix)?;
    println!("Verified {rows} x {cols}, seed {seed}: all leftmost minima match brute force.");
    if rows <= 16 && cols <= 16 {
        if !is_monge(&matrix)? {
            return Err("generated matrix failed Monge check".into());
        }
        println!("Exhaustive Monge check: passed (excluded from search metrics).");
    }
    report("brute_force", &brute);
    report("smawk", &fast);
    if visual {
        let (traced, trace) = smawk_traced(&matrix)?;
        if traced.minima != brute.minima {
            return Err("traced search disagrees with brute force".into());
        }
        let diagram = render_svg(&matrix, &traced, &trace)?;
        let path = Path::new(
            options
                .get("--out")
                .map_or("artifacts/example.svg", String::as_str),
        );
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, diagram.svg)?;
        println!(
            "SVG: {} | rendering_evaluations={} trace_buffer_bytes={}",
            path.display(),
            diagram.evaluations,
            trace.buffer_bytes()
        );
    }
    Ok(())
}
