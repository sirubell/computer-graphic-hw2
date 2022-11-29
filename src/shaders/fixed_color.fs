#version 330 core

uniform vec3 fixed_color;

out vec4 frag_color;


void main()
{
    frag_color = vec4(fixed_color, 1.0);
}
