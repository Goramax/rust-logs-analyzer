use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

fn main() {
    let mut requests_nb: u32 = 0;
    let mut response_codes = HashMap::new();
    let mut ips = HashMap::new();
    let mut analysis_time: Instant = Instant::now();
    let line_count: u64;
    let sample_size: u32;

    println!("Entrez le chemin du fichier à analyser:");
    let filepath = ask_for("Chemin du fichier:");

    loop {
        println!("Entrez la taille de l'échantillon: (entrez 0 pour traiter le fichier entier) ");
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => match input.trim().parse() {
                Ok(num) => {
                    sample_size = num;
                    break;
                }
                Err(_) => {
                    println!("Veuillez entrer un nombre valide.");
                    continue;
                }
            },
            Err(_) => println!("Impossible de lire la ligne"),
        }
    }

    println!("Lecture du nombre de lignes, cela peut prendre un moment...");

    let start_time = Instant::now();
    if sample_size != 0 {
        line_count = sample_size as u64;
    } else {
        line_count = count_lines(&filepath.clone()).unwrap();
    }

    let pb = ProgressBar::new(line_count);
    pb.set_style(ProgressStyle::default_bar()
.template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} ({eta})")
.progress_chars("#>-"));

    // Open the file line by line
    if let Ok(lines) = read_lines(&filepath) {
        analysis_time = Instant::now();
        for line in lines.flatten() {
            let mut parts = line.splitn(2, ' ');
            let first_part = parts.next().unwrap_or_default();

            // Get the IP address from the first part of the line
            *ips.entry(first_part.to_string()).or_insert(0) += 1;

            for word in parts.next().unwrap_or_default().split_whitespace() {
                // Get the first 3 characters of the word and check if they are all numbers
                if word.len() == 3 && word.chars().all(char::is_numeric) {
                    *response_codes.entry(word.to_string()).or_insert(0) += 1;
                    break;
                }
            }

            requests_nb = requests_nb + 1;
            if requests_nb % 5000 == 0 {
                pb.inc(5000);
            }

            if sample_size != 0 && requests_nb >= sample_size {
                break;
            }
        }
    }

    pb.finish_with_message("Analyse terminée.");

    // Get the IP address with the most requests
    let (most_frequent_ip, most_frequent_count) = ips
        .iter()
        .max_by_key(|&(_, &count)| count)
        .map(|(ip, &count)| (ip.clone(), count))
        .unwrap_or((String::new(), 0));

    // Display the results
    println!("Nombre de requêtes: {}", requests_nb);
    // Display the response codes and order descending
    let mut response_codes_vec: Vec<_> = response_codes.iter().collect();
    response_codes_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (code, count) in response_codes_vec {
        println!("Statut HTTP {}: {}", code, count);
    }
    println!("Nombre total d'adresses IP uniques: {}", ips.len());
    println!(
        "Adresse IP la plus fréquente: {} ({} requêtes, {:.2}% du total)",
        most_frequent_ip,
        most_frequent_count,
        (most_frequent_count as f64 / requests_nb as f64) * 100.0
    );
    println!("Temps d'analyse du fichier: {:?}", analysis_time.elapsed());
    println!(
        "Temps total d'exécution du script: {:?}",
        start_time.elapsed()
    );
    println!("Appuyez sur Entrée pour quitter le programme...");
    ask_for("");
}

fn read_lines<P>(filepath: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filepath)?;
    Ok(io::BufReader::new(file).lines())
}

fn count_lines<P>(filepath: P) -> io::Result<u64>
where
    P: AsRef<Path>,
{
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count() as u64)
}

fn ask_for(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Échec de la lecture de l'entrée");
    input.trim().to_string()
}
