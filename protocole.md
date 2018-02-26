Proposition de protocole
========================

Proposition de protocole pour la communication inter-serveurs/clients pour le projet de chat textuel.

Quelques règles pour assurer le bon fonctionnement du protocole :

-   Chaque ligne se termine par les caractères `\r\n`.
-   Un nom d’utilisateur ne peut contenir que des caractères alphanumériques (a-z, A-Z, 0-9).

Connexion au serveur
--------------------

Gestion de l’arrivée des utilisateurs sur le serveur (choix du nom d’utilisateur et notification des clients).

### Requête 1.1

Connexion sans nom d’utilisateur fournit par le client, `<version>` devant être remplacé par la version utilisée par le client (tel que `0.5` ou `1.0`). Client vers serveur.

``` text
PROT <version> CONNECT NEW
```

### Requête 1.2

Connexion au serveur mentionnant le nom d’utilisateur

``` text
PROT <version> CONNECT USER <username>
```

Selon si la connexion est établie avec ce nom d'utilisateur, la [requête 1.4](#requête-14) ou la [requête 1.5](#requête-15) sera renvoyée. Si la requête [requête 1.5](#requête-15) est renvoyée, la [requête 1.2](#requête-12) suivra immédiatement après.

### Requête 1.3

Réponse à cette requête du serveur vers le client (serveur vers client), requête du nom d'utilisateur.

``` text
PROT <version> REQ NAME
```

### Requête 1.4

Réponse à la requête du serveur (client vers serveur), envoi du nom d’utilisateur.

``` text
PROT <version> NAME <username>
```

### Requête 1.5

Réponse du serveur si l'enregistrement du nom d'utilisateur s’est bien déroulé (serveur vers client)

``` text
PROT <version> OK WELCOME
```

### Requête 1.6

Réponse du serveur si l'enregistrement du nom d'utilisateur a rencontré une erreur (nom déjà utilisé,…) (serveur vers client). La [requête 1.2](#requête-12) est envoyée vers le client.

``` text
PROT <version> FAILURE
```

### Requête 1.7

Conjointement à la [requête 1.4](#requête-14), cette requête sera envoyée à tout autre client connecté pour les notifier de la connexion d’un nouvel utilisateur.

``` text
JOIN <username>
```

Déconnexion du serveur
----------------------

Gestion du départ des utilisateurs du serveur

### Requête 2.1

Envoi du client vers le serveur le notifiant de sa déconnexion

``` text
PROT <version> BYE
```

### Requête 2.2

Envoi du serveur vers chaque client de la notification de déconnexion d'un client

``` text
PROT <version> LOGOUT <username>
```

Ping
----

Vérification de la connexion des clients avec le serveur. Chaque minute, la requête [requête 3.1](#requête-31) est envoyée à chaque client qui ont tous trois secondes pour répondre avec la [requête 3.2](#requête-32).

### Requête 3.1

Envoi d’un ping du serveur vers chaque client.

``` text
PROT <version> PING
```

### Requête 3.2

Envoi de la réponse du client au serveur pour la [requête 3.1](#requête-31)

``` text
PROT <version> PONG
```

## Échange de messages
### Échange de messages publics

#### Requête 4.1.1
Envoi depuis le client vers le serveur d’un message public
``` text
PROT <version> MSG <message>
```

#### Requête 4.1.2

Transmission d’un message d’un client vers les autres clients
``` text
PROT <version> FROM <username> MSG <message>
```

### Échange de messages privés

#### Requête 4.2.1

Transmission d’un message d’un client vers un autre client uniquement, spécifié par son nom d’utilisateur (client vers serveur)

``` text
PROT <version> PRIV TO <dest-username> MSG <msg>
```

#### Requête 4.2.2

Transmission d’un message d’un client vers un autre client uniquement, spécifié par son nom d’utilisateur (serveur vers client)

``` text
PROT <version> PRIV FROM <username> MSG <message>
```


### Messages privés

#### Requête 4.2.1

Transmission d’un message d’un client vers un autre client uniquement, spécifié par son nom d’utilisateur (client vers serveur)

``` text
PROT <version> PRIV TO <dest-username> MSG <msg>
```

#### Requête 4.2.2

Transmission d’un message d’un client vers un autre client uniquement, spécifié par son nom d’utilisateur (serveur vers client)

``` text
PROT <version> PRIV FROM <username> MSG <message>
```
