#version 330 core

uniform vec3 light_intensity;

out vec4 frag_color;

void main() {
    frag_color = vec4(light_intensity, 1.0);
}
