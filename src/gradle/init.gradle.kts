import groovy.json.JsonOutput
import org.gradle.api.artifacts.component.ModuleComponentIdentifier
import org.gradle.api.artifacts.result.ResolvedComponentResult

val begin = "GRADLE_CHECKER_BEGIN"
val end = "GRADLE_CHECKER_END"

gradle.projectsEvaluated {
    gradle.rootProject.tasks.register("gradleCheckerInspect") {
        doLast {
            val candidates = gradle.rootProject.allprojects.flatMap { project ->
                project.configurations.filter { it.isCanBeResolved }.map { project to it }
            }
            val mode = gradle.rootProject.providers.gradleProperty("gradleCheckerMode").orElse("list").get()
            val payload: Any = if (mode == "list") {
                mapOf("configurations" to candidates.map { (project, configuration) ->
                    mapOf("project" to project.path, "name" to configuration.name)
                })
            } else {
                val selector = gradle.rootProject.providers.gradleProperty("gradleCheckerConfiguration").get()
                val matches = candidates.filter { (project, configuration) ->
                    val qualified = if (project.path == ":") ":${configuration.name}" else "${project.path}:${configuration.name}"
                    if (selector.startsWith(":")) qualified == selector else configuration.name == selector
                }
                require(matches.size == 1) {
                    if (matches.isEmpty()) "configuration '$selector' was not found"
                    else "configuration '$selector' is ambiguous: " + matches.joinToString { "${it.first.path}:${it.second.name}" }
                }
                val root = matches.single().second.incoming.resolutionResult.root
                val requestedModules = gradle.rootProject.providers.gradleProperty("gradlensModules").orNull?.split(',')?.toSet().orEmpty()
                val components = linkedMapOf<String, Any>()
                fun visit(component: ResolvedComponentResult) {
                    val id = component.id as? ModuleComponentIdentifier ?: return
                    val key = "${id.group}:${id.module}:${id.version}"
                    if (components.containsKey(key)) return
                    val children = component.dependencies.mapNotNull { dependency ->
                        val selected = (dependency as? org.gradle.api.artifacts.result.ResolvedDependencyResult)?.selected
                        val childId = selected?.id as? ModuleComponentIdentifier
                        childId?.let { "${it.group}:${it.module}:${it.version}" }
                    }.distinct().sorted()
                    components[key] = mapOf(
                        "id" to mapOf("module" to mapOf("group" to id.group, "name" to id.module), "version" to id.version),
                        "children" to children,
                        "metadata_urls" to emptyList<String>()
                    )
                    component.dependencies.forEach { dependency ->
                        (dependency as? org.gradle.api.artifacts.result.ResolvedDependencyResult)?.selected?.let(::visit)
                    }
                }
                val directComponents = root.dependencies.mapNotNull { dependency ->
                    (dependency as? org.gradle.api.artifacts.result.ResolvedDependencyResult)?.selected
                }
                val selectedRoots = directComponents.filter { component ->
                    val id = component.id as? ModuleComponentIdentifier
                    id != null && "${id.group}:${id.module}" in requestedModules
                }
                selectedRoots.forEach(::visit)
                val roots = selectedRoots.mapNotNull { component ->
                    val id = component.id as? ModuleComponentIdentifier
                    id?.let { "${it.group}:${it.module}:${it.version}" }
                }.distinct().sorted()
                mapOf("components" to components, "roots" to roots)
            }
            println(begin)
            println(JsonOutput.toJson(payload))
            println(end)
        }
    }
}
