# Robot Object‑Avoidant Path Generation Visualizer

Simulation for FIRST robotics on collision avoidant autonomous path generation around dynamic obstacles.

## What This Is
- A simulation: a virtual environment to experiment with collision‑avoidant path planning.
- A Rust app that generates and visualizes collision‑avoiding paths for a robot on a 2D field.
- Uses a field‑based approach: obstacles create repulsive fields that allow the path to be optimized by following the negative gradient away from obstacles.
- Built with `nannou` for rendering and `nannou_egui` for interactive controls; loads simple 3D shapes from `models/*.stl` to draw obstacles and markers.
- Intended for robotics path planning experiments, such as FIRST Robotics autonomous routines.

## How It Works
- Field and models: The app draws a rectangular field and renders STL models (robot base, cubes, etc.) as wireframes for visibility.
- Obstacles: Each obstacle contributes to a scalar “height” field in the plane. The total field is the sum of all obstacle fields.
- Path generation: A straight, segmented path is seeded between the robot’s start and the target. Each intermediate point gets a height based on nearby obstacles.
- Path optimization: Iteratively adjusts path points by moving them along the negative gradient of the obstacle field and enforcing a minimum obstacle clearance. The process stops when all points are “low enough” and safely distant, or when max iterations are reached.
- Visualization: You can toggle a gradient field overlay, see the generated path, place points along the path, and tune parameters (segments, speeds, resolutions).

## A Bit of Math
This project uses transformed piecewise cosine field functions and gradient‑based optimization to shape a collision‑avoidant path.

- Obstacle field (transformed piecewise cosine): for obstacle $i$ with center $c_i$ and clearance radius $R_{\text{calc},i}$

  $$
  V_i(r) =
  \begin{cases}
    \dfrac{b_i}{2}\,\cos\\left(\dfrac{\pi r}{b_i}\right), & 0 \le r \le R_{\text{calc},i} \\
    0, & r > R_{\text{calc},i}
  \end{cases}
  \quad \text{with } b_i = \pi\,R_{\text{calc},i}.
  $$

  where $r = \|p - c_i\|$ and $R_{\text{calc},i} = r_{\text{obstacle}} + r_{\text{robot}} + r_{\text{buffer}}$ scales the transform so the field vanishes at the clearance boundary.

- Gradient (repulsive direction): letting $\mathbf{e}_r = (p - c_i)/r$ be the radial unit vector,

$$
\nabla V_i(p) = -\,\frac{\pi}{2}\,\sin\!\left(\frac{\pi r}{b_i}\right)\,\mathbf{e}_r.
$$

  In code, this is scaled by an adjustment rate and applied so that points move away from obstacles when accumulating gradients across all obstacles.

- Total field and descent step:

$$
V_{\text{total}}(p) = \sum_i V_i(p), \qquad
\nabla V_{\text{total}}(p) = \sum_i \nabla V_i(p),
$$

  and path points are updated by a repulsive step akin to

$$
p \leftarrow p - \alpha\,\nabla V_{\text{total}}(p),
$$

  with additional clearance enforcement when $r < R_{\text{calc},i}$.

- Convergence criteria: the optimizer stops when point heights fall below a threshold and all points maintain a minimum safe distance from every obstacle, or after a maximum iteration cap.

This formulation provides a smooth, tunable repulsion field and clear geometric semantics for robotics path planning (including FIRST Robotics autonomous path shaping in simulation).

## Controls and UI
In the right panel:
- Toggle `Show Gradient Function` and adjust `X Resolution`, `Y Resolution`, and `Line Resolution`.
- Click `Update Gradient Field` to refresh the overlay.
- Under `Obstacles`:
  - `Create New Obstacle` inputs for name, scale, and position.
  - `Add Obstacle` to place it on the field.
  - Manage existing obstacles: select, move, change radius, `Delete Obstacle`.
- Under `Target Position`:
  - Drag `X` and `Y` to move the goal; this auto‑regenerates and re‑optimizes the path.
  - `Robot Movement`: sliders for `X Velocity`, `Y Velocity`, and `Target Speed`.
- Under `Path Settings`:
  - Toggle `Show Path`.
  - Adjust `Path Segments`.
  - Buttons: `Generate Path`, `Follow Path`, `Place Points Along Path`, `Clear All Path Points`.

## Screenshots / Images

### Simulation Enviornment

`![Enviornment](docs/img/path.png)`

### Obstacle Calculation

`![Moving Obstacle](docs/img/moving_obstacle.gif)`

### Obstacle Interaction

`![Obstacle Interaction](docs/img/obstacle_interaction.gif)`

### Path Recalculation

`![Path Recalculation](docs/img/path_recalculation.gif)`

### Path Following

`![Path Following](docs/img/path_follow.gif)`

## Project Structure
- `src/main.rs` — app entry and UI; rendering and interaction.
- `src/robot.rs` — robot model, path generation, optimization, path following.
- `src/obstacle.rs` — obstacle model; cosine and gaussian field functions and gradients.
- `src/gradient_field.rs` — builds gradient wire overlays from the field function.
- `src/model.rs`, `src/wire.rs`, `src/position.rs`, `src/field.rs`, `src/target_position.rs` — supporting types for geometry, drawing, and state.
- `models/` — STL models used for wireframe visualization.

## Notes and Roadmap
- The cosine field is the default for optimization; gaussian field utilities exist and can be experimented with.
- Path optimization caps at a max iteration count to avoid infinite loops; thresholds and rates are configurable in code.
