width := "768"
samples := "100"
max_depth := "50"

dump SCENE:
    cargo run --release --bin cli -- dump \
        {{SCENE}} > scenes/{{SCENE}}.json

gui:
    cargo run --release --bin gui

render_cover:
    cargo run --release --bin cli -- render \
        -w {{width}} \
        -s {{samples}} \
        -d {{max_depth}} \
        --lookfrom 13,2,3 \
        --lookat 0,0,0 \
        --vup 0,1,0 \
        --vfov 20 \
        --defocus-angle 0 \
        ./scenes/cover.json > image.ppm

render_earth:
    cargo run --release --bin cli -- render \
        -w {{width}} \
        -s {{samples}} \
        -d {{max_depth}} \
        --lookfrom 0,0,12 \
        --lookat 0,0,0 \
        --vup 0,1,0 \
        --vfov 20 \
        --defocus-angle 0 \
        ./scenes/earth.json > image.ppm

render_perlin:
    cargo run --release --bin cli -- render \
        -w {{width}} \
        -s {{samples}} \
        -d {{max_depth}} \
        --lookfrom 13,2,3 \
        --lookat 0,0,0 \
        --vup 0,1,0 \
        --vfov 20 \
        --defocus-angle 0 \
        ./scenes/perlin_spheres.json > image.ppm


render_quads:
    cargo run --release --bin cli -- render \
        -r 1.0 \
        -w {{width}} \
        -s {{samples}} \
        -d {{max_depth}} \
        --lookfrom 0,0,9 \
        --lookat 0,0,0 \
        --vup 0,1,0 \
        --vfov 80 \
        --defocus-angle 0 \
        ./scenes/quads.json > image.ppm