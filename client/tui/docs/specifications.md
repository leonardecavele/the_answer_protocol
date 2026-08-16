# Spécifications de The Answer Protocol (Client TUI)

Ce document décrit l'architecture globale, les règles de conception et l'état des lieux de l'implémentation du client terminal (TUI). Il sert de référence vivante au fur et à mesure de l'évolution du code.

## 1. Règles Strictes (English Codebase)
- **Code et Commentaires en Anglais** : L'intégralité du code source (noms de variables, fonctions, traits) ainsi que tous les commentaires `//` ou docstrings `///` doivent obligatoirement être rédigés en anglais. Seule la documentation de haut niveau et les échanges peuvent être en français.

## 2. Architecture Générale
L'application repose sur un cycle asynchrone central (dans `app.rs`) qui reçoit des événements depuis un canal `mpsc`. Tout est conçu pour être non-bloquant pour l'interface graphique.

- **EventBroker** (`src/events/broker.rs`) : Gère les événements d'entrée (clavier, redimensionnement) et les ticks temporels. Il tourne dans une tâche asynchrone séparée.
- **NetworkManager** (`src/network/manager.rs`) : S'occupe de la communication avec le serveur via `api-client`. Il est instancié *uniquement* lorsque l'utilisateur déclenche une connexion. La tâche asynchrone gère la connexion TCP puis l'authentification logique. Une fois connecté, il agit comme un routeur abstrait (zéro duplication) : il pousse les paquets du serveur (`ServerEvent`) vers le TUI, et écoute un canal interne pour recevoir des ordres encapsulés (`RequestEnvelope`) depuis le TUI afin de les exécuter via l'API client et retourner un `ResponseEnvelope`.
- **Boucle de l'Application** (`src/app/mod.rs`) : Architecturée en module, le `mod.rs` instancie `App` et gère le cycle de dessin et d'attente d'événements. Tous les traitements spécifiques d'événements (Terminal, Network, ServerEvent, ApiResponse) sont délégués à des sous-modules métiers (dossier `src/app/handlers/`) pour éviter d'avoir un fichier centralisé gigantesque. L'interception des erreurs globales d'API s'y fait de manière centralisée. À chaque tour, elle met à jour l'état (`AppState`) et distribue l'événement à la vue active.

**Paradigme Event-Driven Strict :**
Les vues (`LoginView`, `GameView`) ne modifient jamais l'état global directement ni ne gèrent les réponses réseau. Elles se contentent d'émettre des événements ou d'envoyer des requêtes à l'API (`RequestEnvelope`). L'application (`app.rs`) intercepte les retours asynchrones (`ResponseEnvelope`), vérifie s'il y a une erreur pour afficher une notification globale, et en cas de succès, délègue la mutation du `GameState` aux handlers (ex: `handlers/api.rs`). Les vues se contentent de lire cet état muté pour se redessiner.

## 3. Le Système d'États (AppState)
L'état global de l'application est découpé logiquement pour éviter un fichier `app.rs` monolithique. Il est situé dans `src/states/`.

- `UiState` : Concerne ce qui est affiché. Indique la vue actuellement active, gère la pile de notifications et le focus (`GameFocus`).
- `NetworkState` : Concerne l'état de la connexion. IP du serveur, port, statut (déconnecté, tentative, connecté), ping.
- `GameState` : Concerne la partie métier pure. Nom du joueur, santé, liste des joueurs dans la même salle (`room_players`), membres du groupe (`group_members`), compteur de joueurs connectés, historique complet des discussions classé par canaux (`chat_history`), et l'état du dialogue en cours (`active_dialogue`), respectant ainsi le Domain-Driven Design (un dialogue PNJ est une mécanique de jeu, non de l'UI générique).

## 4. Communication Réseau (Zéro Duplication)
Le TUI n'implémente aucun pattern-matching complexe ni n'encapsule ses propres types de commandes. Il se repose entièrement sur la crate `api-client` pour le protocole.
La génération des requêtes et des réponses est entièrement automatisée par la macro `define_api_protocol!` dans `api-client`, qui agit comme source de vérité unique.

L'interaction UI -> API est bâtie sur le principe des **Envelopes** (dossier `src/network/envelopes.rs`) :
1. La zone de texte (Footer) envoie un simple événement texte : `ApplicationEvent::SendRawCommand(String)`.
2. Le cœur `App::update` intercepte cette chaîne et utilise le parseur généré (`ApiRequest::parse`) pour valider la syntaxe sans recourir à des guillemets. En cas d'échec, une `Notification::warning` est affichée.
3. Si le texte est valide, l'application construit une `RequestEnvelope` (l'assignation de l'UUID est proprement encapsulée via `RequestEnvelope::new()`) et l'envoie au channel asynchrone du `NetworkManager`.
4. Le `NetworkManager` exécute la requête de manière abstraite via `client.execute_request(api_request)`.
5. Le résultat est renvoyé sous forme de `ResponseEnvelope` dans la boucle d'événements.
6. L'`App` intercepte l'enveloppe de retour. Si `get_error()` détecte un refus du serveur, l'erreur est affichée globalement dans l'UI. Sinon, le succès est transmis à `handlers/api.rs` pour mettre à jour l'`AppState`.
7. **Automatisations (ex: MOVE -> LOOK)** : Certains retours déclenchent de nouvelles requêtes automatiquement. Par exemple, un retour en succès de `MOVE` déclenche immédiatement l'envoi d'une commande `LOOK` silencieuse pour rafraîchir la salle.

## 5. Le Système de Vues (UI) et Composants (Components)
L'interface utilisateur est architecturée autour de deux concepts :
1. **Les Vues (`AppView`)** : Représentent un écran complet (ex: `LoginView`, `GameView`).
2. **Les Composants (`Component`)** : Des éléments d'interface réutilisables ou superposables.
   - *Composants globaux* : `NotificationComponent`, `EventOverlayComponent` gérés par `App`.
   - *Composants d'interface (Widgets)* : `TextInputComponent`, `ButtonComponent` utilisés à l'intérieur des vues.

Le trait `Component` (dans `src/ui/components/mod.rs`) est l'interface pure de notre UI. Il est minimaliste :
- `draw()` : Méthode pour dessiner le composant.
- `handle_terminal_event()` : Pour réagir aux événements clavier ou souris.
- `is_blocking()` : Pour la gestion de focus des modales.

**L'Architecture Interactive (Pattern Decorator)** :
Pour les widgets interactifs (boutons, champs textes), nous utilisons un wrapper générique `Interactive<T>`. 
L'objectif est d'éliminer totalement le boilerplate (`last_area`, détection de clic) du code métier :
1. Le développeur crée une structure de composant pure (ex: `ButtonComponent`) qui n'a **aucune notion** de son emplacement visuel (`Rect`).
2. Ce composant implémente le trait `InteractiveComponent` qui fournit une méthode `handle_terminal_event` avec un flag `is_hovered` magique calculé en amont.
3. Dans la vue, le composant est enveloppé (ex: `Interactive::new(ButtonComponent::new("..."))`).
4. C'est l'instance `Interactive<T>` (qui implémente `Component`) qui intercepte l'aire de dessin dans `draw()`, sauvegarde cette position secrètement, calcule les collisions de clics, et délègue l'événement au sous-composant. Le composant métier est ainsi libéré de toute gestion mathématique ou d'état visuel "mort".

L'instance `App` gère la vue courante, ainsi que des composants globaux (Notifications et Overlay d'événements) dessinés *par-dessus* la vue courante via des `ratatui::widgets::Clear`.

### 5.1 Architecture de la GameView
La `GameView` est la vue principale du jeu. Elle agit comme un orchestrateur et divise l'écran en plusieurs panneaux modulaires :
- **Header** : Informations dynamiques sur l'emplacement actuel (Nom de la salle encadré et sa description complète récupérés via la commande `look`).
- **Left Panel** : Divisé verticalement en 3 listes (Room Players, Room NPCs, Group Members). Les PNJs sont colorés selon leur type via le Manifest, et le nom du joueur est mis en surbrillance.
- **Center Panel** : Historique dynamique des actions du joueur (résultats d'attaques, dialogues, etc.) formaté dans un paragraphe défilant automatiquement.
- **Right Panel** : Rendu visuel de la salle actuelle ou du PNJ ciblé. Ce panneau est **borderless** (sans bordure ni titre) afin de maximiser l'espace pour l'image. L'image est redimensionnée dynamiquement pour occuper toujours 100% de la dimension limitante tout en conservant ses proportions (`Resize::Scale`), et est parfaitement centrée horizontalement et verticalement. En cas d'erreur de chargement, le texte de repli est pré-découpé et centré mathématiquement. Un badge `[ FOCUS ]` discret est superposé uniquement si ce panneau est ciblé par le système global.
- **Footer** : Zone de saisie des commandes interactives. Contient un `Interactive<TextInputComponent>` avec focus qui intercepte la touche Entrée.
- **Chat Overlay** : Panneau flottant (affichable/masquable avec `F1`) dédié à la discussion, utilisant un formatage de paragraphe qui garantit que les messages ne dépassent pas la bordure.
- **NpcActionPopup** : Modale contextuelle qui s'ouvre lorsqu'on sélectionne un PNJ dans le `Left Panel`. Elle liste les actions possibles définies dans le manifest (ex: `TALK`, `ATTACK`) et intercepte les commandes pour les envoyer au serveur.
- **DialoguePopupComponent** : Modale d'affichage façon "RPG" superposée en bas de l'écran. Elle affiche progressivement (effet machine à écrire via `on_tick()`) la réponse d'un PNJ suite à une commande `TALK`. La touche Entrée accélère l'affichage ou ferme la modale. Un tag serveur `[end of dialogue]` permet de signifier la fin de la discussion.

### 5.2 Système de Focus Global et Navigation
L'application intègre un système robuste de gestion de focus (`GameFocus`) enregistré dans le `UiState`.
Ce système est conçu pour éviter que des touches destinées aux actions en jeu ne s'écrivent par erreur dans la console (Footer).
1. **Cycle Tabulaire** : L'utilisateur peut appuyer sur `Tab` pour naviguer (cycle naturel Input -> NpcList -> RightPanel) et sur `Shift+Tab` (BackTab) pour reculer.
2. **Raccourcis Intelligents (Entrée)** : Un appui sur `Entrée` dans un Footer vide bascule automatiquement le focus sur l'Image. Un appui sur `Entrée` sur l'Image bascule automatiquement sur la liste des NPCs. Cela permet une navigation "One-key" extrêmement fluide entre les panneaux.
3. **Support de la Souris** : Les clics de souris sont interceptés et délégués au système de focus. Pour éviter la duplication de code et garantir une fiabilité mathématique, un helper global `is_mouse_in_rect(col, row, area)` est implémenté dans `src/ui/utils.rs`. Il valide précisément les clics internes, par exemple en ciblant uniquement le tiers de la liste "Room NPCs" (ce qui sélectionne également le PNJ sous le curseur), ou la zone d'image, réattribuant instantanément le focus au composant ciblé.

## 6. Gestion des Événements et du Temps Absolu

### 6.1 Algorithme de Rendu Textuel (textwrap)
Afin d'assurer un autoscroll "pixel-perfect" et d'empêcher les débordements de texte sur des écrans étroits, l'application s'appuie sur la librairie `textwrap` et des helpers internes (`wrap_str_to_lines`, `wrap_slice_to_lines`). Contrairement au wrapper natif de Ratatui qui masque l'information de lignes visuelles, cette approche pré-calcule les sauts de lignes pour déterminer la véritable hauteur (height) et le décalage (scroll offset) d'un texte. Cela est systématiquement utilisé dans le Header, le Center Panel, le Right Panel et les notifications.

### 6.2 Système de Notifications
Les notifications sont empilables visuellement et dynamiques en hauteur (le texte revient à la ligne proprement), limitées en largeur à 30% de l'écran, et limitées en nombre par `MAX_VISIBLE_NOTIFICATIONS`. 
Le `NotificationComponent` mémorise l'emplacement exact de chaque notification active. L'utilisateur peut cliquer sur n'importe quelle notification visible pour forcer sa fermeture instantanée.
**Builder Pattern & ID Ciblés** : La création de notification utilise un pattern élégant (`Notification::info("msg").with_id("id").with_duration(10000)`). Cela permet de garder un code métier propre tout en offrant la possibilité de fournir un ID optionnel fixe. Cela permet au code central (`app.rs`) de cibler et détruire une notification spécifique (comme un message "Connecting...") dès que la tâche asynchrone est résolue, évitant la superposition d'informations obsolètes.

### 6.2 Système de Manifest (Data Mapping)
Le serveur ne transmettant que des identifiants (ex: `"npc_001"`, `"item_42"`), le client s'appuie sur une base de données locale pour l'affichage visuel.
- Ce "Manifest" est chargé de manière synchrone au démarrage via le fichier de constante globale `ASSETS_PATH_MANIFEST` (`assets/manifest.json`).
- **Il se limite strictement au cosmétique et à l'interaction locale** : il ne contient pas de données métiers redondantes. Par exemple, pour les salles, le manifest ne stocke que l'`image_path`. Les descriptions et noms textuels sont obtenus directement depuis le backend (commande `look`).
- Il traduit les IDs en noms d'affichage lisibles pour les PNJ et fournit un `NpcType` (Enemy, QuestGiver, Dialogue) utilisé pour colorer dynamiquement les entités dans le `Left Panel`.
- Il fournit le champ optionnel `actions` (tableau de strings) pour définir le menu dynamique de la popup `NpcActionPopup` (ex: `["talk", "attack"]`).
- Le parsing est fail-safe (`serde(default)`) : si le fichier manque ou est invalide, le système utilise des données vides par défaut sans jamais crasher.

## 7. Gestion des Erreurs
Toutes les erreurs sont centralisées via l'enum `ApplicationError` dans `src/errors.rs`, propulsé par la crate `thiserror`.

## 8. Configuration & Constantes
L'application évite l'anti-pattern d'un fichier global `constants.rs` fourre-tout.
1. **Localisation Forte** : Les constantes internes (historiques, durée d'affichage, tick rate) sont définies au plus proche de leur domaine d'utilisation (dans `states/ui.rs`, `events/broker.rs`, etc.).
2. **Interface en Ligne de Commande (CLI)** : L'adresse IP et le port du serveur ne sont plus codés en dur. Ils sont passés comme arguments au lancement de l'exécutable grâce à la librairie `clap` (`cargo run -- --ip 1.1.1.1 --port 8080`). La fonction `--help` est générée automatiquement.
