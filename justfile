dump SCENE:
    cargo run --release -- dump \
        {{SCENE}} > scenes/{{SCENE}}.json

render_cover:
    cargo run --release -- render \
        -w 768 \
        -s 100 \
        -d 50 \
        --lookfrom 13,2,3 \
        --lookat 0,0,0 \
        --vup 0,1,0 \
        --vfov 20 \
        --defocus-angle 0 \
        ./scenes/cover.json > image.ppm

render_earth:
    cargo run --release -- render \
        -w 768 \
        -s 100 \
        -d 50 \
        --lookfrom 0,0,12 \
        --lookat 0,0,0 \
        --vup 0,1,0 \
        --vfov 20 \
        --defocus-angle 0 \
        ./scenes/earth.json > image.ppm

render_perlin:
    cargo run --release -- render \
        -w 768 \
        -s 100 \
        -d 50 \
        --lookfrom 13,2,3 \
        --lookat 0,0,0 \
        --vup 0,1,0 \
        --vfov 20 \
        --defocus-angle 0 \
        ./scenes/perlin_spheres.json > image.ppm