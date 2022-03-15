#version 460

layout (location = 0) in vec2 speed;

layout (location = 0) out vec4 fragColor;

layout (binding = 0) uniform Uniform {
    ivec2 window;
    ivec2 camera;
    float zoom;
    float alpha;

    vec3 color_base;
    vec3 color_fast;
    float color_ratio;
} info;

void main() {
    float speed_total = sqrt(speed.x * speed.x + speed.y * speed.y);

    fragColor = vec4(0.5, 0.5, 0.5, 1.0);
    fragColor = vec4(1.0, 1.0 - speed_total / 100000.0, 1.0 - speed_total / 100000.0, info.alpha);

    float factor_speed = min(info.color_ratio * (speed_total / 100000.0), 1.0);

    vec3 color_final = mix(info.color_base, info.color_fast, factor_speed);
    fragColor = vec4(color_final, info.alpha);
}