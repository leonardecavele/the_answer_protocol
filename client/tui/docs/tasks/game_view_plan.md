# Plan d'Implémentation de la GameView

Ce document sert de feuille de route pour construire la vue principale du jeu (la `GameView`) de manière itérative. Chaque étape sera implémentée et validée avant de passer à la suivante.

## Itération 1 : Squelette Principal & Architecture Composants
**Objectif :** Poser les fondations de l'affichage en divisant l'écran et en respectant l'architecture par composants (`Component`).
- Transformation de `tui/src/ui/views/game.rs` en module : `tui/src/ui/views/game/mod.rs`.
- Création du sous-dossier `tui/src/ui/views/game/components/` pour y héberger les composants spécifiques à la vue du jeu :
  - `header.rs` -> `HeaderComponent` (fixe en haut)
  - `footer.rs` -> `FooterComponent` (fixe en bas)
  - `left_panel.rs` -> `LeftPanelComponent` (colonnes de listes)
  - `center_panel.rs` -> `CenterPanelComponent` (zone centrale d'historique)
  - `right_panel.rs` -> `RightPanelComponent` (image)
  - `chat_overlay.rs` -> `ChatOverlayComponent` (overlay en bas à droite)
- Utilisation de `ratatui::layout::Layout` dans `GameView::draw` pour découper l'écran et déléguer l'affichage à ces composants.
- Affichage de `Block` simples avec bordures (`Borders::ALL`) au sein de ces composants pour matérialiser les zones vides.
- Implémentation du toggle de base pour afficher/masquer le `ChatOverlayComponent` (ex: touche F1).
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**

## Itération 2 : Footer Interactif (Input de Commande)
**Objectif :** Permettre au joueur de taper des commandes.
- [x] Intégration du composant existant `TextInputComponent` dans la zone **Footer**.
- [x] Gestion de l'interception clavier pour ce composant au sein de `GameView::handle_terminal_event`.
- [x] Appui sur `Entrée` : Vidage de l'input et émission d'un log système factice pour vérifier le bon fonctionnement.
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**

## Itération 3 : Left Panel (Informations Locales)
**Objectif :** Afficher les entités présentes.
- [x] Découpage vertical du panneau de gauche en 3 sous-zones égales :
  1. `Room Players` (Joueurs présents)
  2. `Room NPCs` (PNJs)
  3. `Group Members` (Notre groupe)
- [x] Affichage de listes (`ratatui::widgets::List`) lisant le `GameState`.
- [x] Intégration d'un système de `Manifest` local pour mapper les identifiants en données UI (noms et couleurs).
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**

## Itération 4 : Right Panel (Placeholder Image)
**Objectif :** Réserver l'espace visuel pour la vue de la pièce.
- [x] Création d'un block dans le panneau de droite.
- [x] Intégration de `ratatui-image` avec un Cache et `StatefulImage` pour afficher les NPCs et Salles dynamiquement.
- [x] Fallbacks avec texte statique traduit en anglais (ex: "Cluster 6").
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**
## Itération 4.5 : Système de Commandes (Parser API)
**Objectif :** Intégrer un parser natif sans librairie tierce pour interpréter les inputs joueurs.
- [x] Utilisation du polymorphisme via `from_str` dans le trait `Command`.
- [x] Remplacement du routing manuel par une macro génératrice (`define_api_protocol!`).
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**

## Itération 4.6 : Nettoyage et Refonte Réseau (Footer)
**Objectif :** Nettoyer l'architecture de communication TUI <-> Serveur.
- [x] Isoler le `FooterComponent` des connaissances réseau (`SendRawCommand`).
- [x] Déplacer le parsing asynchrone au cœur de `App::update`.
- [x] Encapsuler la génération UUID derrière `RequestEnvelope::new()`.
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**
## Itération 5 : Center Panel (Historique des Actions)
**Objectif :** Afficher le flux d'événements et d'actions du joueur.
- [x] Ajout de `action_logs` dans `GameState` et interception des réponses `Talk` et `Attack`.
- [x] Affichage d'une zone de texte au centre avec autoscroll.
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**

## Itération 5.1 : Layout & Immersion (Header et Right Panel)
**Objectif :** Améliorer l'affichage et l'immersion, et déléguer les données dynamiques de salle au serveur.
- [x] Le Header affiche le nom et la description de la salle de manière dynamique (réponse du serveur à un `LOOK`).
- [x] Le Right Panel n'a plus de bordure : l'image (salle ou PNJ) prend toute la place disponible.
- [x] Le wrapping (`Wrap`) est appliqué partout où c'est nécessaire pour empêcher les textes de fuir de l'écran.
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**

## Itération 5.2 : Stabilité du Texte et Autoscroll (textwrap)
**Objectif :** Remplacer le système natif de wrapping pour résoudre définitivement les fuites de texte.
- [x] Intégration de la librairie `textwrap`.
- [x] Ajout de helpers (`wrap_str_to_lines`, `wrap_slice_to_lines`).
- [x] Calcul exact du scroll pour `CenterPanelComponent` et `EventOverlayComponent`.

## Itération 5.3 : Pixel-Perfect Layout (Image & Textes)
**Objectif :** Finitions visuelles avancées sur le Right Panel et les overlays.
- [x] Éradication de la hauteur artificielle pour les textes de repli du Right Panel (centrage vertical dynamique).
- [x] Affichage intelligent de l'image (type `object-fit: contain`) : utilise toujours 100% de la largeur ou de la hauteur sans déformer les proportions, avec double centrage automatique.
- [x] Migration de l'Overlay de notifications vers `textwrap` avec hauteur calculée dynamiquement au caractère près.
- [x] **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**

## Itération 6 : Enrichissement du Chat Overlay
**Objectif :** Rendre l'overlay de chat (créé en itération 1) fonctionnel.
- Interception des inputs spécifiques au chat lorsque l'overlay est actif.
- Récupération de l'historique complet des discussions via `GameState::chat_history`.
- Gestion du défilement des messages (scrolling) dans ce panneau réduit.
- **Mise à jour du document des spécifications (`specifications.md`) pour acter cette itération.**
