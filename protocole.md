Proposition de protocole
========================

Proposition de protocole pour la communication inter-serveurs/clients pour le projet de chat textuel.

Quelques règles pour assurer le bon fonctionnement du protocole :

-   Chaque ligne se termine par les caractères `\r\n`.
-   Un nom d’utilisateur ne peut contenir que des caractères alphanumériques (a-z, A-Z, 0-9).
-   Un nom d’utilisateur contient de 1 (un) à 20 (vingt) caractères.
-   Un message contient de 1 (un) à 2000 (deux mille) caractères (hors protocole)

Liste d’abbréviations :

-   "C" : client unique
-   "A" : ensemble des clients
-   "S" : serveur

Général
-------

### Requête 0.1

S->C

Requête non comprise par le serveur

``` text
BAD REQ
```

Connexion au serveur
--------------------

Gestion de l’arrivée des utilisateurs sur le serveur (choix du nom d’utilisateur et notification des clients).

### Requête 1.1

C->S

Connexion sans nom d’utilisateur fournit par le client, `<version>` devant être remplacé par la version utilisée par le client (tel que `0.5` ou `1.0`).

``` text
PROT <version> CONNECT NEW
```

### Requête 1.2

C->S

Connexion au serveur mentionnant le nom d’utilisateur

``` text
PROT <version> CONNECT USER <username>
```

Selon si la connexion est établie avec ce nom d'utilisateur, la [requête 1.4](#requête-14) ou la [requête 1.5](#requête-15) sera renvoyée. Si la requête [requête 1.5](#requête-15) est renvoyée, la [requête 1.2](#requête-12) suivra immédiatement après.

### Requête 1.3

S->C

Réponse à cette requête du serveur vers le client (serveur vers client), requête du nom d'utilisateur.

``` text
NAME REQ
```

### Requête 1.4

C->S

Réponse à la requête du serveur (client vers serveur), envoi du nom d’utilisateur.

``` text
NAME <username>
```

### Requête 1.5

S->C

Réponse du serveur si l'enregistrement du nom d'utilisateur s’est bien déroulé, immédiatement suivi par la [requête 1.9](#requête-18)

``` text
NAME OK
```

### Requête 1.6

S->C

Réponse du serveur si l'enregistrement du nom d'utilisateur a rencontré une erreur (nom déjà utilisé,…) (serveur vers client).

``` text
NAME FAILURE
```

### Requête 1.7

S->A

Conjointement à la [requête 1.4](#requête-14), cette requête sera envoyée à tout autre client connecté pour les notifier de la connexion d’un nouvel utilisateur.

``` text
JOIN <username>
```

### Requête 1.8

S->C

Requête confirmant au client sa connexion

``` text
WELCOME
```

### Requête 1.9

S->C

Réponse du serveur en cas de version de protocole différente

```text
BAD PROT
```

Déconnexion du serveur
----------------------

Gestion du départ des utilisateurs du serveur

### Requête 2.1

C->S
S->C

Du client vers le serveur : notification de déconnexion du client au serveur.

Du serveur vers le client : confirmation de déconnexion du client depuis le serveur.

``` text
BYE
```

### Requête 2.2

S->A

Notification aux clients de la déconnexion d’un autre client.

``` text
LOGOUT <username>
```

Ping
----

Vérification de la connexion des clients avec le serveur. Chaque minute, la requête [requête 3.1](#requête-31) est envoyée à chaque client qui ont tous trois secondes pour répondre avec la [requête 3.2](#requête-32).

### Requête 3.1

S->A, C->S

Envoi d’un ping du serveur vers chaque client ou d’un client vers le serveur.

``` text
PING
```

### Requête 3.2

C->S, S->C

Envoi de la réponse du client au serveur ou du serveurs au client pour la [requête 3.1](#requête-31)

``` text
PONG
```

## Échange de messages

### Échange de messages publics

#### Requête 4.1.1

C->S

Envoi depuis le client vers le serveur d’un message public

``` text
MSG <message>
```

#### Requête 4.1.2

S->A

Transmission d’un message d’un client vers les autres clients

``` text
FROM <username> MSG <message>
```

### Interactions salon de chat

#### Requête 4.2.1

C->S

Demande du client pour recevoir la liste des participants connectés

```text
REQ CLIENTS
```

#### Requête 4.2.2

S->C

Réponse du client à la [requête 4.2.1](#requête-421) transmettant au client la liste des autres clients connectés
```text
LIST CLIENTS <nombre de clients> <noms clients séparés par un espace>
```

Comme mentionné au début de ce document, aucun caractère blanc n’est autorisé dans les pseudonymes afin qu’il n’y ait pas de collision avec le protocole.
