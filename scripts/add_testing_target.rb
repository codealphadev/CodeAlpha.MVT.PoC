require 'xcodeproj'

testing_target_name = 'NiceTests'
name_of_target_to_test = 'hello2'

project_path = './hello2.xcodeproj'
project = Xcodeproj::Project.open(project_path)

target_for_testing = project.targets.find { |target| target.name == name_of_target_to_test }

# Add test target with XCTest framework
test_target = project.new(Xcodeproj::Project::PBXNativeTarget)
project.targets << test_target
test_target.name = testing_target_name
test_target.product_name = testing_target_name
test_target.product_type = 'com.apple.product-type.bundle.unit-test'
test_target.build_configuration_list = Xcodeproj::Project::ProjectHelper.configuration_list(project, :macos)

product_ref = project.products_group.new_reference(testing_target_name + '.xctest', :built_products)
product_ref.include_in_index = '0'
product_ref.set_explicit_file_type
test_target.product_reference = product_ref

test_target.build_phases << project.new(Xcodeproj::Project::PBXSourcesBuildPhase)
test_target.build_phases << project.new(Xcodeproj::Project::PBXFrameworksBuildPhase)
test_target.build_phases << project.new(Xcodeproj::Project::PBXResourcesBuildPhase)

# # Add xctest framework manually because it's in a weird folder.
# group = project.frameworks_group['iOS'] || project.frameworks_group.new_group('iOS')
# path = "Library/Frameworks/XCTest.framework"
# unless ref = group.find_file_by_path(path)
#   ref = group.new_file(path, :developer_dir)
# end
# test_target.frameworks_build_phase.add_file_reference(ref, true)

# Add other targets.
test_target.add_system_frameworks(['UIKit', 'Foundation'])

# Add target dependency.
test_target.add_dependency(target_for_testing)

# Special test build configs.
test_target.build_configuration_list.set_setting('WRAPPER_EXTENSION', 'xctest')
test_target.build_configuration_list.set_setting('BUNDLE_LOADER', "$(BUILT_PRODUCTS_DIR)/#{target_for_testing.name}.app/#{target_for_testing.name}")
test_target.build_configuration_list.set_setting('TEST_HOST', '$(BUNDLE_LOADER)')

test_target.build_configuration_list.build_configurations.each do |bc|
    bc.build_settings['FRAMEWORK_SEARCH_PATHS'] = [
        '$(SDKROOT)/Developer/Library/Frameworks',
        '$(inherited)',
        '$(DEVELOPER_FRAMEWORKS_DIR)']
end

# TODO normal build configs for that target.
# TODO add test classes to that target.