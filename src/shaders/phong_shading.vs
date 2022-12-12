#version 330 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 normal;

uniform mat4 world_matrix;
uniform mat4 normal_matrix;
uniform mat4 mvp;

out vec3 i_position;
out vec3 i_normal;

void main() {
    i_position = vec3(world_matrix * vec4(position, 1.0));
    i_normal = vec3(normal_matrix * vec4(normal ,0.0));
    gl_Position = mvp * vec4(position, 1.0);
}
