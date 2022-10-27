require 'xcodeproj'
require 'optparse'
options = {}
OptionParser.new do |parser|
    parser.on("--target-to-test TARGET", "The name of the XCode target to test") do |v|
        options[:target_to_test] = v
    end
    parser.on("--path-to-xcodeproj PATH", "The absolute path to the .xcodeproj file") do |v|
        options[:path_to_xcodeproj] = v
    end
    parser.on("--path-to-test-directory PATH", "The absolute path to the test directory") do |v|
        options[:path_to_test_directory] = v
    end
end.parse!

if options[:target_to_test].nil?
    puts "Please provide --target-to-test"
    exit
end
if options[:path_to_xcodeproj].nil?
    puts "Please provide --path-to-xcodeproj"
    exit
end
if options[:path_to_test_directory].nil?
    puts "Please provide --path-to-test-directory" # TODO: Add trailing slash if missing
    exit
end

# TODO: Example call ruby add_testing_target.rb --target-to-test hello2 --path-to-xcodeproj /Users/daniel/Documents/HelloWorldApp/hello3/hello2.xcodeproj --path-to-test-directory /Users/daniel/Documents/HelloWorldApp/hello3/PretzlTests/

testing_target_name = 'PretzlTests'
name_of_target_to_test = options[:target_to_test]
test_file_name = testing_target_name + ".swift"
test_file_path = options[:path_to_test_directory] + test_file_name # TODO: Convert relative path to absolute

project_path = options[:path_to_xcodeproj]
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
#test_target.add_system_frameworks(['UIKit', 'Foundation'])

# Add target dependency.
test_target.add_dependency(target_for_testing)


# My settings
test_target.build_configuration_list.set_setting('BUNDLE_LOADER', "$(TEST_HOST)")
test_target.build_configuration_list.set_setting('CODE_SIGN_STYLE', 'Automatic')
test_target.build_configuration_list.set_setting('CURRENT_PROJECT_VERSION', 1)
test_target.build_configuration_list.set_setting('GENERATE_INFOPLIST_FILE', "YES")
test_target.build_configuration_list.set_setting('MARKETING_VERSION', "1.0") # Do we need this?
test_target.build_configuration_list.set_setting('PRODUCT_BUNDLE_IDENTIFIER', "com.codealpha.ConfidenceTests")
test_target.build_configuration_list.set_setting('PRODUCT_NAME', "$(TARGET_NAME)")
test_target.build_configuration_list.set_setting('SWIFT_EMIT_LOC_STRINGS', 'NO')
test_target.build_configuration_list.set_setting('SWIFT_VERSION', '5.0')
test_target.build_configuration_list.set_setting('TEST_HOST', "$(BUILT_PRODUCTS_DIR)/#{target_for_testing.name}.app/Contents/MacOS/#{target_for_testing.name}")


# Special test build configs.
# test_target.build_configuration_list.set_setting('WRAPPER_EXTENSION', 'xctest')
# test_target.build_configuration_list.set_setting('BUNDLE_LOADER', "$(BUILT_PRODUCTS_DIR)/#{target_for_testing.name}.app/#{target_for_testing.name}")
# test_target.build_configuration_list.set_setting('TEST_HOST', '$(BUNDLE_LOADER)')

# test_target.build_configuration_list.build_configurations.each do |bc|
#     bc.build_settings['FRAMEWORK_SEARCH_PATHS'] = [
#         '$(SDKROOT)/Developer/Library/Frameworks',
#         '$(inherited)',
#         '$(DEVELOPER_FRAMEWORKS_DIR)']
# end

# TODO normal build configs for that target.


# Add new test group
test_group = project.main_group.find_subpath(testing_target_name, true)
unless test_group 
    test_group = project.main_group.new_group(testing_target_name)
end
new_file = test_group.new_file(test_file_path)
new_file.include_in_index = '0'

test_target.add_file_references([new_file])
project.save()