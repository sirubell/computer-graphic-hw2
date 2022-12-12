#version 330 core

in vec3 i_position;
in vec3 i_normal;

out vec4 frag_color;

uniform vec3 camera_pos;

uniform vec3 ka;
uniform vec3 kd;
uniform vec3 ks;
uniform float ns;

uniform vec3 ambient_light;

uniform vec3 point_light_pos;
uniform vec3 point_light_intensity;

uniform vec3 spot_light_pos;
uniform vec3 spot_light_intensity;
uniform vec3 spot_light_dir;
uniform float cutoff_start;
uniform float total_width;

uniform vec3 dir_light_dir;
uniform vec3 dir_light_radiance;

vec3 diffuse(vec3 kd, vec3 i, vec3 n, vec3 light_dir);
vec3 specular(vec3 ks, vec3 i, vec3 n, vec3 camera_dir, vec3 reflect_dir, float ns);
vec3 point_light(vec3 point_light_pos, vec3 position, vec3 normal, vec3 camera_dir);
vec3 spot_light(vec3 spot_light_pos, vec3 position, vec3 normal, vec3 camera_dir);
vec3 dir_light(vec3 dir_light_dir, vec3 normal, vec3 camera_dir);


vec3 diffuse(vec3 kd, vec3 i, vec3 n, vec3 light_dir) {
    return kd * i * max(dot(n, light_dir), 0.0);
}

vec3 specular(vec3 ks, vec3 i, vec3 n, vec3 camera_dir, vec3 reflect_dir, float ns) {
    return ks * i * pow(max(dot(camera_dir, reflect_dir), 0.0), ns);
}

vec3 point_light(vec3 position, vec3 normal, vec3 camera_dir) {
    vec3 light_dir = normalize(point_light_pos - position);
    vec3 reflect_dir = reflect(-light_dir, normal);

    float distance = length(point_light_pos - position);
    float attenuation = 1.0 / (distance * distance);
    vec3 intensity = point_light_intensity * attenuation;

    vec3 diffuse = diffuse(kd, intensity, normal, light_dir);
    vec3 specular = specular(ks, intensity, normal, camera_dir, reflect_dir, ns);

    return diffuse + specular;
}

vec3 spot_light(vec3 position, vec3 normal, vec3 camera_dir) {
    vec3 light_dir = normalize(spot_light_pos - position);
    vec3 reflect_dir = reflect(-light_dir, normal);

    float cos_theta = dot(light_dir, normalize(-spot_light_dir));
    float epsilon = cos(radians(cutoff_start)) - cos(radians(total_width));

    float distance = length(spot_light_pos - position);
    float attenuation = 1.0 / (distance * distance);
    vec3 intensity = spot_light_intensity * clamp((cos_theta - cos(radians(total_width))) / epsilon, 0.0, 1.0)  * attenuation;

    vec3 diffuse = diffuse(kd, intensity, normal, light_dir);
    vec3 specular = specular(ks, intensity, normal, camera_dir, reflect_dir, ns);

    return diffuse + specular;
}

vec3 dir_light(vec3 normal, vec3 camera_dir) {
    vec3 light_dir = normalize(-dir_light_dir);
    vec3 reflect_dir = reflect(-light_dir, normal);
    
    vec3 diffuse = diffuse(kd, dir_light_radiance, normal, light_dir);
    vec3 specular = specular(ks, dir_light_radiance, normal, camera_dir, reflect_dir, ns);

    return diffuse + specular;
}

void main() {
    vec3 normal = normalize(i_normal);
    vec3 camera_dir = normalize(camera_pos - i_position);
    
    vec3 i_color = ka * ambient_light;
    i_color += point_light(i_position, normal, camera_dir);
    i_color += spot_light(i_position, normal, camera_dir);
    i_color += dir_light(normal, camera_dir);

    frag_color = vec4(i_color, 1.0);
}
