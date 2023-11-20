pipeline {
    agent { label 'docker' }
    environment {
        CARGO_HOME = "${WORKSPACE}/.cargo"
    }
    stages {
        stage('Check') {
            parallel {
                stage('Check tessa4d') {
                    agent {
                        docker {
                            // Reusing bevy-build for make and rustup components
                            image 'gitea.okaymonkey.com/cicd/bevy-build:latest'
                            alwaysPull true
                            reuseNode true
                        }
                    }
                    steps {
                        sh 'make check-tessa'
                    }
                }
                stage('Check tessa4d-gdext') {
                    agent {
                        docker {
                            image 'gitea.okaymonkey.com/cicd/gdext-rs-build:4.1.3-latest'
                            alwaysPull true
                            reuseNode true
                        }
                    }
                    steps {
                        sh 'make check-gdext'
                        sh 'make build-gdext'
                    }
                }
                stage('Check tessa4d-bevy') {
                    agent {
                        docker {
                            image 'gitea.okaymonkey.com/cicd/bevy-build:latest'
                            alwaysPull true
                            reuseNode true
                        }
                    }
                    steps {
                        sh 'make check-bevy'
                    }
                }
            }
        }
        stage('Integration Tests') {
            parallel {
                stage('itest tessa4d-gdext') {
                    agent {
                        docker {
                            image 'gitea.okaymonkey.com/cicd/godot-ci:4.1.3-latest'
                            alwaysPull true
                            reuseNode true
                        }
                    }
                    steps {
                        // Xvfb allows running X without an acutal display
                        // so that we can take screenshots and test rendering in itests.
                        sh 'pulseaudio & xvfb-run -s "-screen 0 1280x1024x24" make itest-gdext'
                    }
                }
            }
        }
    }
    post {
        always {
            archiveArtifacts artifacts: 'itest/tessa4d-gdext/tests/screenshots/*.png', fingerprint: true
        }
    }
}
