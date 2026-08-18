# Itération 7 : Interactions et Logique Métier

Ce plan regroupe l'ensemble des règles d'interactions et de confort de jeu (UI/UX), révisées et validées.

## 1. Système de Focus Global (Navigation Clavier/Souris)
- **Enumération `GameFocus`** : Gère de manière globale le focus actif (`Input`, `RightPanel`, `NpcList`) dans `UiState`.
- **Navigation Naturelle** : La touche **Tab** fait avancer le focus dans l'ordre de lecture (Input -> NpcList -> RightPanel -> Input). La combinaison **Shift+Tab** (BackTab) parcourt le cycle en sens inverse.
- **Clique Souris** : Un clic de souris sur l'une des zones (limité de manière précise à la liste "Room NPCs", à l'image borderless, ou au footer) donne instantanément le focus au composant cliqué grâce au helper centralisé `is_mouse_in_rect`.
- **Indicateurs Visuels** :
  - Le panneau "Room NPCs" gagne une bordure colorée (jaune) lorsqu'il a le focus.
  - Les éléments normaux de la liste sont mis en valeur avec `Color::White` pour un contraste optimal.
  - Le panneau de l'image ("Right Panel"), qui est borderless, affiche un badge discret `[ FOCUS ]` en surimpression en haut à droite uniquement lorsqu'il est actif.

## 2. Déplacements Dynamiques (Right Panel)
- **Objectif** : Afficher les directions disponibles et s'y déplacer facilement.
- **Détails techniques** : 
  - La réponse `LOOK` fournit une liste `exits` (ex: `{"NORTH": "room_2"}`). Cette liste sera sauvegardée dans le `GameState`.
  - Le `RightPanelComponent` dessinera dynamiquement en surimpression des étiquettes (ex: `Up [North]`, `Right [East]`) correspondant aux sorties existantes, uniquement lorsque le panneau aura le focus.
  - L'appui sur une flèche déclenchera la commande de déplacement appropriée (ex: `ApplicationEvent::SendRawCommand("MOVE NORTH")`) à condition que l'exit correspondant soit valide et que le panneau soit sélectionné.

## 3. Popup Contextuelle sur les PNJs (Left Panel)
- **Objectif** : Rendre la liste des PNJs interactive pour choisir une action.
- **Détails techniques** : 
  - Lorsque la liste "Room NPCs" a le focus, les touches `Up/Down` permettent de sélectionner un PNJ avec un système de **boucle à l'infini** (passer du dernier au premier et inversement).
  - L'appui sur `Enter` déclenche l'affichage d'un nouveau composant `NpcActionPopup` par-dessus l'interface (mode modale bloquante).
  - La popup lit le champ `actions` du PNJ directement depuis le `manifest.json`.
  - Toutes les actions sont converties et affichées en **MAJUSCULES** dynamiquement.
  - La modale possède son propre curseur (avec navigation infinie). Appuyer sur l'action émet la commande textuelle correspondante.

## 4. Automatisations et Formatage (Dialogues & Mouvements)
- **Noms vs IDs** : Dans le panneau central, lors du formatage des logs d'actions (`TALK`, `ATTACK`), les IDs techniques (ex: `npc_001`) seront traduits à la volée en noms lisibles via le Manifest.
- **Mouvement Automatique** : Lors d'un succès de l'action `MOVE` (`ApiResponse::Move`), le code va générer et envoyer automatiquement une requête `LOOK` au serveur sans intervention du joueur pour rafraîchir l'environnement visuel.
- **Fin de Conversation** : Lors d'un `TALK`, le code vérifiera si la réponse du serveur contient la chaîne `"[end of dialog]"`. Si c'est le cas (ou si le joueur change de salle avec `MOVE`), le PNJ actuel (`focused_entity_id`) sera désélectionné pour que l'image de la salle reprenne sa place.

## 5. Hub d'Information (Header & Groupes)
- **Box Groupe** : Écoute des événements asynchrones (`ServerEvent::Group`) dans le gestionnaire réseau pour maintenir à jour le `GameState::group_members` en temps réel.
- **Header (Statistiques)** :
  - Affichage du nombre de joueurs connectés (via `ServerEvent::Stats`).
  - Affichage des Points de Vie (HP) du joueur. Obtenus via une commande `STATUS` silencieuse à la connexion et mis à jour via les retours de la commande `ATTACK`.
