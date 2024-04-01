# Analyse de fichier de logs Rust vs PHP

## Objectif
L'objectif de ce script est d'obtenir des statistiques concernant le nombre total de requêtes, ainsi que des statistiques pour chaque code de retour.
Il faut aussi récupérer le nombre total d'adresses uniques et l'adresse ip ayant fait le plus de requêtes ainsi que le pourcentage de ses requêtes par rapport au total.
Le temps d'exécution du script doit aussi apparaître.

Le fichier étant très volumineux, il faut trouver un moyen de l'analyser ligne par ligne afin de ne pas surcharger la RAM.

### Structure d'une ligne de log

#### Exemple
xxx.xx.xx.xxx - - [24/Mar/2024:21:20:21 +0000] "GET /en/std-exchanges/21866-moteur-100mm-iec-6nm-ip54-bmh1002t16a2apr.html HTTP/1.1" 200 71133 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_1) AppleWebKit/600.2.5 (KHTML, like Gecko) Version/8.0.2 Safari/600.2.5 (Amazonbot/0.1; +https://developer.amazon.com/support/amazonbot)"

#### Découpage
184.30.41.xxx -> Adresse ip source

[24/Mar/2024:21:20:21 +0000] -> DateTime de la requête

"GET /en/std-exchanges/21866-moteur-100mm-iec-6nm-ip54-bmh1002t16a2apr.html HTTP/1.1" -> Type de requête, url, protocole

200 -> Code de retour

## Version mono-thread
> [!NOTE]
> J'ai ajouté en bonus une barre de chargement pour suivre l'avancement du traitement du fichier.
> Afin de déterminer la taille de la barre de chargement, je dois récupérer le nombre de lignes du fichier, ce qui prend un peu de temps avant.
> 
> Dans l'affichage des statistiques, je différencie `Temps d'analyse du fichier`, qui correspond au temps de traitement du fichier sans le comptage du nombre de lignes, et `Temps total d'exécution du script`, qui correspond au temps total de traitement du fichier avec le comptage du nombre de lignes.

- Le script demande à l'utilisateur de renseigner le **path du fichier de logs** à analyser.
- Le script demande si l'utilisateur souhaite **choisir un échantillon** (correpond au x premières lignes du fichier). L'utilisateur doit rentrer 0 pour analyser tout le fichier.
- Le script lit le fichier ligne par ligne et incrémente la variable contenant le nombre total de lignes.
- Le script lit le fichier ligne par ligne et incrémente les variables contenant les statistiques.
  - Pour cette partie, j'ai choisi de découper la chaine de caractères en 2 au premier espace. Je récupère la première partie pour l'adresse ip.
  - Dans la deuxième partie je re découpe sur les espaces et pour chaque élément, le premier sur lequel je tombe qui est composé de 3 caractères numériques est le code de retour.
  - J'ai choisi cette méthode car l'utilisation de regex augmentait significativement le temps de traitement du fichier.
- Le script affiche les statistiques et le temps d'exécution.


## Version multi-thread
> [!WARNING]
> J'ai essayé de faire une ouverture ligne par ligne via `BufReader` mais je n'ai pas réussi à le faire fonctionner, le traitement ralentissait et bloquait à la moitié du fichier. N'ayant pas eu le temps de débugger, j'ai opté pour une ouverture du fichier en entier dans la RAM. Cela m'a permis d'obtenir un résultat.

> [!CAUTION]
> Attention à avoir au moins 7Go (taille du fichier ssl_access_log) de RAM de libre pour exécuter le script multi-thread.

- Le script demande à l'utilisateur de renseigner le **path du fichier de logs** à analyser.
- Le script demande si l'utilisateur souhaite **choisir un échantillon** (correpond au x premières lignes du fichier). L'utilisateur doit rentrer 0 pour analyser tout le fichier.
- Le script demande à l'utilisateur de renseigner le **nombre de threads** à utiliser pour le traitement du fichier.
- Contrairement à la version mono-thread, le script charge le fichier en entier dans la RAM.
- Le script découpe le fichier en plusieurs parties égales pour chaque thread.
  - Même méthode de découpage que pour la version mono-thread.
- Chaque thread traite sa partie du fichier et incrémente les variables contenant les statistiques.
- Le script affiche les statistiques et le temps d'exécution.


## Exécuter les scripts

### Installation de Rust
[Installer Rust](https://www.rust-lang.org/fr/tools/install)

### Installer les dépendances
Se rendre dans le dossier multi_threaded ou single_threaded et exécuter la commande suivante :
```bash 
cargo install --path .
```

### Exécuter le script
Se rendre dans le dossier multi_threaded ou single_threaded et exécuter la commande suivante :
```bash 
cargo run
```
Ou bien build le projet et utiliser l'exécutable généré dans le dossier target/debug :
```bash
cargo build
```

## Analyse et comparaison des résultats

### Tableau comparatif des résultats obtenus
| PHP                                                                             | Rust Mono-Thread                                                                    | Rust Multi-Thread (60 threads)                                                                  |
|---------------------------------------------------------------------------------|--------------------------------------------------------------------------------|--------------------------------------------------------------------------------|
| Nombre total de requêtes: 35094647                                              | Nombre de requêtes: 35094647                                                   | Nombre de requêtes: 35094647                                                   |
| Statut HTTP 503 : 17                                                            | Statut HTTP 200: 33124246                                                      | Statut HTTP 200: 33124246                                                      |
| Statut HTTP 302 : 1180893                                                       | Statut HTTP 302: 1180893                                                       | Statut HTTP 302: 1180893                                                       |
| Statut HTTP 500 : 83306                                                         | Statut HTTP 404: 494340                                                        | Statut HTTP 404: 494340                                                        |
| Statut HTTP 200 : 33124246                                                      | Statut HTTP 403: 95140                                                         | Statut HTTP 403: 95140                                                         |
| Statut HTTP 301 : 2230                                                          | Statut HTTP 500: 83306                                                         | Statut HTTP 500: 83306                                                         |
| Statut HTTP 404 : 494340                                                        | Statut HTTP 304: 77295                                                         | Statut HTTP 304: 77295                                                         |
| Statut HTTP 304 : 77295                                                         | Statut HTTP 499: 18802                                                         | Statut HTTP 499: 18802                                                         |
| Statut HTTP 400 : 2575                                                          | Statut HTTP 502: 10549                                                         | Statut HTTP 502: 10549                                                         |
| Statut HTTP 403 : 95140                                                         | Statut HTTP 206: 2657                                                          | Statut HTTP 206: 2657                                                          |
| Statut HTTP 499 : 18802                                                         | Statut HTTP 400: 2575                                                          | Statut HTTP 400: 2575                                                          |
| Statut HTTP 206 : 2657                                                          | Statut HTTP 301: 2230                                                          | Statut HTTP 301: 2230                                                          |
| Statut HTTP 405 : 1076                                                          | Statut HTTP 401: 1381                                                          | Statut HTTP 401: 1381                                                          |
| Statut HTTP 401 : 1381                                                          | Statut HTTP 405: 1076                                                          | Statut HTTP 405: 1076                                                          |
| Statut HTTP 504 : 131                                                           | Statut HTTP 504: 131                                                           | Statut HTTP 504: 131                                                           |
| Statut HTTP 502 : 10549                                                         | Statut HTTP 503: 17                                                            | Statut HTTP 503: 17                                                            |
| Statut HTTP 204 : 9                                                             | Statut HTTP 204: 9                                                             | Statut HTTP 204: 9                                                             |
| Nombre total d'adresses IP uniques : 13701                                      | Nombre total d'adresses IP uniques: 13701                                      | Nombre total d'adresses IP uniques: 13701                                      |
| Adresse IP la plus fréquente : 92.122.54.14 (7297725 requêtes, 20.79% du total) | Adresse IP la plus fréquente: 92.122.54.14 (7297725 requêtes, 20.79% du total) | Adresse IP la plus fréquente: 92.122.54.14 (7297725 requêtes, 20.79% du total) |
| Temps d'exécution du script: 31.918715000153 secondes.                          | Temps d'analyse du fichier: 275.0785403s                                       | Temps d'analyse du fichier: 122.8567785s                                       |

### Specs de la machine utilisée pour les tests ci-dessus
- Processeur: Intel Core i7-9750H
- RAM: 16Go 2666MHz
- SSD: Samsung MZVLB512HAJQ-00000
- OS: Windows 11

### Analyse des temps d'exécution
Nous pouvons observer que le script PHP est le plus rapide, exécuté en 32 secondes contre 275 secondes pour le script Rust mono-thread et 123 secondes pour le script Rust multi-thread (60 threads).
Le script PHP est donc 8.6 fois plus rapide que le script Rust mono-thread et 3.8 fois plus rapide que le script Rust multi-thread.


## Conclusion
Dans ce cas précis, le script PHP est beaucoup plus efficace que les scripts Rust.
Dans la théorie, Rust en multi-thread devrait être plus rapide que PHP, ce qui n'est pas le cas ici, cela peut être dû à la façon dont je procède pour analyser le fichier qui n'est pas assez optimisée, ou bien à une mauvaise utilisation du multi-threading de ma part, à cause de mon peu d'expérience en Rust.