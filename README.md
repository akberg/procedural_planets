# Procedural Planets

Final project in TDT4230 Graphics and visualization. Procedurally generated planets with varying noise and colours. Focus has been on applying Level of Detail techniques, like scaling, moving coordinate system, and generating terrain with higher resolution depending on distance to planet. Detail level allows flying around in a solar system, as well as landing on and exploring the vast emptiness of each planet.

### Running

The project is written in [Rust](https://rust-lang.org) and requires Cargo. Run with `cargo run`

### Controls

* **`W A S D`**, **`shift`**, **`space`**: Movement
* **`Up`**, **`Down`**: Increase or decrease movement speed
* **`F`**: Toggle free float or anchor to center of gravity of closest planet
* **`I`**: Toggle GUI
* **`M`**: Cycle polygon modes (fill, lines, points)

## State of the project


![Starting to get some eye candy 2](report/images/solar-system-from-outer-rim.png)
![Starting to get some eye candy](report/images/correctly-lit0.png)
![Starting to get some eye candy](report/images/view-from-sun1.png)
![Starting to get some eye candy](report/images/view-from-blue1.png)
![Starting to get some eye candy](report/images/blue-from-moon.png)

## Behind the scene

![Generate a cubesphere, vertices should be quite evenly distributed](report/images/cubesphere-wf.png)
The cubesphere. Not perfect, but well suited for my usecase.
![Computed normals, but struggling with seams](report/images/cubesphere-w-noise-normal-seam.png)
Adding noise to the sphere makes the foundation for a planet.
![Apply noise and add an additional sphere as an ocean](report/images/red-planet-w-ocean.png)
An extra, low-poly sphere does the job as an ocean.
![Playing with adding a height dependent colour map](report/images/planet-w-cheated-heightmap.png)
Adding height dependent colour scheme.
![LoD, though buggy, is making progress](report/images/lod-wireframes.png)
The varying detail level is shown clearly in wireframe mode.
