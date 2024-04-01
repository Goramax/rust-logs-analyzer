# Rust Logs Analyzer

## Objectif

L'objectif de ce script est d'obtenir des statistiques concernant le nombre total de requêtes, ainsi que des statistiques pour chaque code de retour.
Il faut aussi récupérer le nombre total d'adresses uniques et l'adresse ip ayant fait le plus de requêtes ainsi que le pourcentage de ses requêtes par rapport au total.
Le temps d'exécution du script doit aussi apparaître.

Le fichier étant très volumineux, il faut trouver un moyen de l'analyser ligne par ligne afin de ne pas surcharger la RAM.

## Structure d'une ligne de log

### Exemple
184.30.41.xxx - - [24/Mar/2024:21:20:21 +0000] "GET /en/std-exchanges/21866-moteur-100mm-iec-6nm-ip54-bmh1002t16a2apr.html HTTP/1.1" 200 71133 "-" "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_10_1) AppleWebKit/600.2.5 (KHTML, like Gecko) Version/8.0.2 Safari/600.2.5 (Amazonbot/0.1; +https://developer.amazon.com/support/amazonbot)"

### Découpage
184.30.41.xxx -> Adresse ip source

[24/Mar/2024:21:20:21 +0000] -> DateTime de la requête

"GET /en/std-exchanges/21866-moteur-100mm-iec-6nm-ip54-bmh1002t16a2apr.html HTTP/1.1" -> Type de requête, url, protocole

200 -> Code de retour

**Seules les informations ci-dessus sont utiles pour l'exercice.**

## Démarche d'analyse des logs

<u>Variables :</u>

**requestsNb** (int) : Nombre total de requêtes (utilisé aussi pour le nb de lignes)

**ips** ([{ip : nb}]) : Tableau de clé - valeur avec l'ip comme clé et le nombre de requêtes comme valeur

**responseCodes** ([{code : nb}]) : Tableau de clé - valeur avec comme clé chaque code de retour rencontré et en valeur le nombre de fois que ce code a été rencontré

**startTime** (int) : Timestamp initialisée au lancement du script


<u>Fonctionnement :</u>

La constante startTime est initialisée avec le Timestamp actuel

```
Pour chaque ligne :
    avec des regex on récupère l'ip, et le code de retour
        Si l'ip ne se trouve pas dans le tableau, on l'ajoute et on ajoute 1 à sa valeur
        Si le code de retour ne se trouve pas dans le tableau, on l'ajoute et on ajoute 1 à sa valeur
    On ajoute 1 à lastLine

On affiche les stats
```

<u>Notes :</u>

Dès que j'intégrais une regex, le temps d'exécution augmentait significativement. J'ai alors opté pour un découpage de la chaine de caractère au niveau des espaces pour prendre la première partie en tant qu'ip et vérifier la première succession de 3 chiffres trouvée dans la suite de la chaine en tant que code de retour.