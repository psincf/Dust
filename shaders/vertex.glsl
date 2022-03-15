#version 460

layout (location = 0) in ivec2 position;
layout (location = 1) in vec2 speed_in;

layout (location = 0) out vec2 speed;

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
    ivec2 position_relative = position.xy - info.camera;

    //gl_Position = vec4(float(position.x) / float(window.x), float(position.y) / float(window.y), 0.0, 1.0);
    //gl_Position.xy = gl_Position.xy / 10000.0;

    gl_Position = vec4(float(position_relative.x) / float(info.window.x), float(position_relative.y) / float(-info.window.y), 0.0, 1.0);
    gl_Position.xy = gl_Position.xy / info.zoom;

    speed = speed_in;
}