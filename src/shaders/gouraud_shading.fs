#version 330 core

in vec3 i_light_color;

out vec4 frag_color;


void main()
{
    frag_color = vec4(i_light_color, 1.0);
}
