#version 460

layout (location = 0) in vec2 speed;
layout (location = 1) in float gravity;

layout (location = 0) out vec4 fragColor;

void main() {
    float speed_total = sqrt(speed.x * speed.x + speed.y * speed.y);

    fragColor = vec4(gravity / 1000000.0, 0.5, 0.5, 1.0);
    if (gravity > 0.0) {
        fragColor = vec4(1.0, 1.0, 0.2, 1.0);
    } else {
        fragColor = vec4(0.2, 0.2, 1.0, 1.0);
    }

    vec2 point_coord = (gl_PointCoord - vec2(0.5, 0.5)) * 2.0;

    if ((point_coord.x * point_coord.x + point_coord.y * point_coord.y) > 1.0) {
        fragColor.w = 0.0;
    } else {
        fragColor.w = 0.5;
    }

    //fragColor.w = 1.0 - (point_coord.x * point_coord.x + point_coord.y * point_coord.y);
}