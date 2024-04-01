use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

fn main() {
    let requests_nb = Arc::new(Mutex::new(0));
    let ips = Arc::new(Mutex::new(HashMap::<String, u32>::new()));
    let response_codes = Arc::new(Mutex::new(HashMap::<String, u32>::new()));
    let filepath = ask_for("Entrez le chemin du fichier à analyser:");
    let sample_size: usize = ask_for("Entrez la taille de l'échantillon: (entrez 0 pour traiter le fichier entier)").parse().unwrap_or(0);
    let thread_count: usize = ask_for("Entrez le nombre de threads à utiliser: ").parse().unwrap_or(1);

    println!("Lecture du nombre de lignes (monothread), cela peut prendre un moment...(merci de surveiller la ram)");

    let file = File::open(&filepath).expect("Impossible d'ouvrir le fichier");
    let start_time: Instant = Instant::now();
    let reader = BufReader::new(file);
    let lines: Vec<String> = if sample_size > 0 {
        reader.lines().filter_map(Result::ok).take(sample_size).collect()
    } else {
        reader.lines().filter_map(Result::ok).collect()
    };
    let line_count = lines.len();

    println!("Début de l'analyse du fichier...");
    let analysis_time = Instant::now();

    let pb = ProgressBar::new(line_count as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({eta})")
        .progress_chars("#>-"));
    let pb = Arc::new(pb);
    pb.set_position(0);

    let mut handlers = Vec::with_capacity(thread_count);

    for i in 0..thread_count {
        let start_line = i * line_count / thread_count;
        let end_line = (i + 1) * line_count / thread_count;
        let thread_lines = lines[start_line..end_line].to_vec();
        let requests_nb = Arc::clone(&requests_nb);
        let ips = Arc::clone(&ips);
        let response_codes = Arc::clone(&response_codes);
        let progress = Arc::clone(&pb);

        let handle = thread::spawn(move || {
            process_lines(thread_lines, requests_nb, ips, response_codes);
            progress.inc(end_line as u64 - start_line as u64);
        });

        handlers.push(handle);
    }

    for handler in handlers {
        handler.join().unwrap();
    }

    pb.finish_with_message("Analyse terminée.");

    // empty ram by dropping the lines vector
    println!("Libération de la mémoire...");
    drop(lines);

    display_results(&requests_nb, &ips, &response_codes);
    println!("Temps d'analyse du fichier: {:?}", analysis_time.elapsed());
    println!("Temps total d'exécution du script: {:?}", start_time.elapsed());

    println!("Appuyez sur Entrée pour quitter.");
    ask_for("");

}

fn ask_for(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Échec de la lecture de l'entrée");
    input.trim().to_string()
}

fn process_lines(lines: Vec<String>, requests_nb: Arc<Mutex<u32>>, ips: Arc<Mutex<HashMap<String, u32>>>, response_codes: Arc<Mutex<HashMap<String, u32>>>) {
    for line in lines {
        for word in line.split_whitespace() {
            // Get the IP address from the first part of the line
            if let Ok(_ip) = word.parse::<std::net::IpAddr>() {
                let mut ips_lock = ips.lock().unwrap();
                *ips_lock.entry(word.to_string()).or_insert(0) += 1;
                continue;
            }

            // Get the first 3 characters of the word and check if they are all numbers
            if word.len() == 3 && word.chars().all(char::is_numeric) {
                let mut response_codes_lock = response_codes.lock().unwrap();
                *response_codes_lock.entry(word.to_string()).or_insert(0) += 1;
                break;
            }
        }

        let mut num_requests = requests_nb.lock().unwrap();
        *num_requests += 1;

        drop(line);
    }
}

fn display_results(requests_nb: &Arc<Mutex<u32>>, ips: &Arc<Mutex<HashMap<String, u32>>>, response_codes: &Arc<Mutex<HashMap<String, u32>>>) {
    let num_requests = requests_nb.lock().unwrap();
    println!("Nombre de requêtes: {}", num_requests);

    let response_codes = response_codes.lock().unwrap();
    let mut codes_vec: Vec<_> = response_codes.iter().collect();
    codes_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (code, &count) in codes_vec {
        println!("Statut HTTP {}: {}", code, count);
    }

    let ips = ips.lock().unwrap();
    println!("Nombre total d'adresses IP uniques: {}", ips.len());
    if let Some((ip, &count)) = ips.iter().max_by_key(|&(_, &count)| count) {
        println!("Adresse IP la plus fréquente: {} ({} requêtes, {:.2}% du total)", ip, count, (count as f64 / *num_requests as f64) * 100.0);
    }

}
