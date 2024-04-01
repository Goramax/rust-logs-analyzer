use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() {
    let start_time = Instant::now();
    let filepath = ask_for_filepath();
    let sample_size = ask_for_sample_size();
    let num_threads = ask_for_thread_count();

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Failed to build global thread pool");

    let analysis_time = Instant::now();

    let (line_count, lines) = if sample_size != 0 {
        read_sample_lines(&filepath, sample_size).expect("Failed to read sample lines")
    } else {
        let count = count_lines(&filepath).expect("Failed to count lines in file");
        (count, read_lines(&filepath).expect("Failed to read lines"))
    };

    let pb = ProgressBar::new(line_count);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({eta})")
        .progress_chars("#>-"));

    let requests_nb = Arc::new(Mutex::new(0u32));
    let response_codes = Arc::new(Mutex::new(HashMap::new()));
    let ips = Arc::new(Mutex::new(HashMap::new()));

    lines.into_par_iter().for_each_with(pb, |pb, line| {
        let parts: Vec<&str> = line.splitn(2, ' ').collect();
        let ip = parts.get(0).cloned().unwrap_or_default();

        let ips_lock = Arc::clone(&ips);
        let mut ips = ips_lock.lock().unwrap();
        *ips.entry(ip.to_string()).or_insert(0) += 1;

        if let Some(words) = parts.get(1) {
            for word in words.split_whitespace() {
                if word.len() == 3 && word.chars().all(char::is_numeric) {
                    let response_codes_lock = Arc::clone(&response_codes);
                    let mut response_codes = response_codes_lock.lock().unwrap();
                    *response_codes.entry(word.to_string()).or_insert(0) += 1;
                    break;
                }
            }
        }

        let requests_nb_lock = Arc::clone(&requests_nb);
        let mut num_requests = requests_nb_lock.lock().unwrap();
        *num_requests += 1;
        pb.inc(1);
    });

    display_results(&ips, &response_codes, &requests_nb);

    println!("Temps d'analyse du fichier: {:?}", analysis_time.elapsed());
    println!("Temps total d'exécution du script: {:?}", start_time.elapsed());

    pause();
}

fn ask_for_filepath() -> String {
    loop {
        println!("Entrez le chemin du fichier à analyser:");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            return input.trim().to_string();
        } else {
            println!("Impossible de lire la ligne");
        }
    }
}

fn ask_for_sample_size() -> usize {
    loop {
        println!("Entrez la taille de l'échantillon: (entrez 0 pour traiter le fichier entier)");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if let Ok(num) = input.trim().parse() {
                return num;
            } else {
                println!("Veuillez entrer un nombre valide.");
            }
        } else {
            println!("Impossible de lire la ligne");
        }
    }
}

fn ask_for_thread_count() -> usize {
    loop {
        println!("Entrez le nombre de threads à utiliser:");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            if let Ok(num) = input.trim().parse() {
                return num;
            } else {
                println!("Veuillez entrer un nombre valide.");
            }
        } else {
            println!("Impossible de lire la ligne");
        }
    }
}

fn read_lines<P>(filepath: P) -> io::Result<Vec<String>>
where
    P: AsRef<Path>,
{
    BufReader::new(File::open(filepath)?)
        .lines()
        .collect()
}

fn read_sample_lines<P>(filepath: P, sample_size: usize) -> io::Result<(u64, Vec<String>)>
where
    P: AsRef<Path>,
{
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let lines: Vec<_> = reader.lines().take(sample_size).collect::<Result<_, _>>()?;
    Ok((lines.len() as u64, lines))
}

fn count_lines<P>(filepath: P) -> io::Result<u64>
where
    P: AsRef<Path>,
{
    Ok(BufReader::new(File::open(filepath)?).lines().count() as u64)
}

fn display_results(ips: &Arc<Mutex<HashMap<String, u32>>>, response_codes: &Arc<Mutex<HashMap<String, u32>>>, requests_nb: &Arc<Mutex<u32>>) {
    let ips = ips.lock().unwrap();
    let response_codes = response_codes.lock().unwrap();
    let requests_nb = requests_nb.lock().unwrap();

    println!("Nombre de requêtes: {}", requests_nb);
    let mut codes_vec: Vec<_> = response_codes.iter().collect();
    codes_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (code, count) in codes_vec {
        println!("Statut HTTP {}: {}", code, count);
    }

    println!("Nombre total d'adresses IP uniques: {}", ips.len());
    if let Some((ip, &count)) = ips.iter().max_by_key(|&(_, &count)| count) {
        println!(
            "Adresse IP la plus fréquente: {} ({} requêtes, {:.2}% du total)",
            ip,
            count,
            (count as f64 / *requests_nb as f64) * 100.0
        );
    }
}

fn pause() {
    println!("Appuyez sur Entrée pour quitter le programme...");
    let mut _input = String::new();
    let _ = io::stdin().read_line(&mut _input);
}
