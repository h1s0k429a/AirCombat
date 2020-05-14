use crate::player::Player;
use crate::GameState;
use gdnative::*;

#[derive(PartialEq)]
enum State {
    Loading,
    Running,
    GameOver,
}
#[derive(NativeClass)]
#[inherit(Node2D)]
pub struct GameScene {
    state: State,
    enemy_obj: Option<PackedScene>,
    player: Option<Node2D>,
    cam: Option<Camera2D>,
    stage_label: Option<Label>,
}

#[methods]
impl GameScene {
    fn _init(_owner: gdnative::Node2D) -> Self {
        GameScene {
            state: State::Loading,
            enemy_obj: None,
            player: None,
            cam: None,
            stage_label: None,
        }
    }

    #[export]
    unsafe fn _ready(&mut self, owner: gdnative::Node2D) {
        if let Some(node) = &mut owner.get_node(NodePath::from_str("./Triggers/Area2D")) {
            let godot_object = &owner.to_object();
            node.connect(
                "area_entered".into(),
                Some(*godot_object),
                "_on_area2d_area_entered".into(),
                VariantArray::new(),
                0,
            )
            .expect("Couldn't connect triggers/area2d");
        }

        let rust_game_state: Instance<GameState> = owner
            .get_tree()
            .and_then(|tree| tree.get_root())
            .and_then(|root| root.get_node(NodePath::from_str("./rustGameState")))
            .and_then(|node| Instance::try_from_base(node))
            .expect("Failed to get game state instance");

        let label_text = rust_game_state
            .map_mut(|gs, _| format!("Stage {}", gs.current_stage))
            .expect("Couldn't build label text");

        self.stage_label = owner
            .get_node(NodePath::from_str("./Label"))
            .and_then(|node| node.cast::<Label>());
        self.stage_label
            .map(|mut sl| sl.set_text(label_text.into()));

        let mut animation_player = owner
            .get_node(NodePath::from_str("./AnimationPlayer"))
            .and_then(|node| node.cast::<AnimationPlayer>())
            .expect("Couldn't find anmiation player");
        animation_player.play("Stage Display".into(), -1.0, 1.0, false);
    }

    #[export]
    unsafe fn _process(&self, owner: Node2D, delta: f64) {
        let rust_game_state: Instance<GameState> = owner
            .get_tree()
            .and_then(|tree| tree.get_root())
            .and_then(|root| root.get_node(NodePath::from_str("./rustGameState")))
            .and_then(|node| Instance::try_from_base(node))
            .expect("Failed to get game state instance");

        let mut hud_kills = owner
            .get_node(NodePath::from_str("./HUD/Kills"))
            .and_then(|node| node.cast::<Label>())
            .expect("HUD Kills canvas could not be found");

        let hud_text = rust_game_state
            .map_mut(|gs, _| format!("Kills: {}", gs.kills))
            .expect("Couldn't create hud text");
        hud_kills.set_text(hud_text.into())
    }

    #[export]
    unsafe fn start_animation_done(&mut self, mut owner: Node2D) {
        self.stage_label.map(|mut label| label.set_visible(false));

        let mut resource_loader = ResourceLoader::godot_singleton();
        let mut player = resource_loader
            .load("res://PlayerRoot.tscn".into(), "".into(), false)
            .and_then(|res| res.cast::<PackedScene>())
            .and_then(|packed_scene| packed_scene.instance(0))
            .and_then(|scene| scene.cast::<Node2D>())
            .expect("Could not load player scene");

        player.set_position(euclid::Vector2D::new(300.0, 720.0 / 2.0));

        let mut cam = Camera2D::new();
        cam.set_position(euclid::Vector2D::new(360.0, 0.0));
        cam.make_current();
        player.add_child(Some(cam.to_node()), false);

        owner.add_child(Some(player.to_node()), false);
        self.spawn_enemies(owner);
        self.state = State::Running;
        self.player = Some(player);
    }

    #[export]
    unsafe fn _on_area2d_area_entered(&self, owner: Node2D, area: Area2D) {
        let rust_game_state: Instance<GameState> = owner
            .get_tree()
            .and_then(|tree| tree.get_root())
            .and_then(|root| root.get_node(NodePath::from_str("./rustGameState")))
            .and_then(|node| Instance::try_from_base(node))
            .expect("Failed to get game state instance");

        if area.get_collision_layer_bit(4) {
            if self.state == State::Running {
                let player_instance: Instance<Player> = self
                    .player
                    .and_then(|pl| Instance::try_from_base(pl))
                    .expect("Could not covert player to player instance!");
                player_instance
                    .map_mut(|pi, _| pi.speed = 0)
                    .expect("Couldn't set player speed!");

                rust_game_state
                    .map_mut(|gs, _| gs.advance_to_next_stage())
                    .expect("Could not advance to next stage!");

                let mut tree = owner.get_tree().expect("Could not load tree");
                tree.reload_current_scene()
                    .expect("Could not reload current scene");
            }
        }
    }

    pub unsafe fn player_died(&mut self) {
        let player_instance: Instance<Player> = self
            .player
            .and_then(|pl| Instance::try_from_base(pl))
            .expect("Could not covert player to player instance!");

        if let Some(player) = self.player {
            for var in player.get_children().iter() {
                let mut child = Node::from_variant(var).expect("Could not convert child to node");
                child.queue_free();
            }
            if let Some(mut owner) = player.get_owner() {
                owner.remove_child(Some(player.to_node()));
                if let Some(mut label) = owner
                    .get_node(NodePath::from_str("./Label"))
                    .and_then(|node| node.cast::<Label>())
                {
                    label.set_text("Game Over".into());
                    label.set_visible(true);
                    label.set_position(
                        euclid::Vector2D::new(1280.0 / 2.0 - 200.0, 720.0 / 2.0),
                        false,
                    );
                }
            }
        }
        self.state = State::GameOver;
    }

    fn spawn_enemies(&self, owner: Node2D) {}
}
