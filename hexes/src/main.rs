mod systems;
mod consts;
mod map;
mod components;
mod entity_creator;

use components::{
    Spawner,
    Transform,
};

use map::{
    Map,
};

use vermarine_lib::{
    rendering::{
        RenderingWorkloadCreator,
        RenderingWorkloadSystems,
        Drawables,
        draw_buffer::{
            DrawBuffer,
        },
    },
    tetra::{
        self,
        ContextBuilder,
        State,
        Context,
        graphics::{
            Camera,
            self,
            Color,
        },
        input::{
            InputContext,
        },
    },
    shipyard::{
        self,
        *,
    },
    hexmap::{
        Axial,
    },
};

fn main() -> tetra::Result {
    ContextBuilder::new("Hexes", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(Game::new)
}

pub struct Game {
    world: World,
}

impl Game {
    pub fn new(ctx: &mut Context) -> tetra::Result<Self> {
        let mut world = World::new();

        world.add_unique(Map::new());
        world.add_unique((*ctx.input_context()).clone());
        world.add_unique_non_send_sync(Drawables::new(ctx).unwrap());

        world.run(|mut all_storages| {
            entity_creator::create_base(Axial::new(10, 5), &mut all_storages);
        });

        world.entity_builder()
            .with(Spawner::new(120))
            .with(Transform::new(Axial::new(-5, -7)))
            .build();

        world
            .add_rendering_workload(ctx)
            .with_rendering_systems()
            .with_system(system!(systems::draw_hex_map))
            .with_system(system!(systems::draw_agent_paths))
            .with_system(system!(systems::draw_entities))
            .build();

        Ok(Game {
            world,
        })
    }
}

impl State for Game {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        let input_ctx = ctx.input_context();
        self.world.run(|mut ctx: UniqueViewMut<InputContext>| {
            *ctx = (*input_ctx).clone();
        });

        self.world.run(systems::move_camera);
        self.world.run(systems::update_hex_map);
        self.world.run(systems::move_agents);
        self.world.run(systems::spawn_agents);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.world.run_workload("Rendering");
        self.world.run(|mut camera: UniqueViewMut<Camera>, mut draw_buff: UniqueViewMut<DrawBuffer>| {
            camera.update();
            draw_buff.transform_mat = camera.as_matrix();
        });
        self.world.run_with_data(DrawBuffer::flush, ctx);

        Ok(())
    }
}